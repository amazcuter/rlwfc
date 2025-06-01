# WFC系统算法部分 Rust重写设计文档

**作者**: amazcuter  
**版本**: 1.0  
**日期**: 2025-01-25  

## 📋 迁移概述

本文档详细规划了WFC系统核心算法模块（`WFCManager.h`）到Rust的完整迁移策略。这是整个WFC系统的核心组件，包含算法逻辑、状态管理、冲突处理等关键功能。

## 🔧 技术基础：图结构和方向识别

### 无向连接的技术实现

**重要概念说明**：WFC算法需要在网格上进行双向的约束传播，因此所有网格连接在逻辑上都是**无向的**（双向可达）。我们使用petgraph有向图和单向边是一种**技术手段**，用于通过边创建顺序来识别方向信息。

#### 核心设计原理

1. **WFC网格本质**：所有相邻单元格都是双向连通的，约束可以在任意方向传播
2. **有向图的作用**：仅用于通过边创建顺序标记和识别方向信息
3. **边对实现**：每个逻辑无向连接用两条相对的有向边表示
4. **方向识别**：利用petgraph的邻居返回逆序特性和预定义索引映射

#### 对WFC算法的影响

```rust
// WFC约束传播需要双向访问所有邻居
fn propagate_to_neighbors(&mut self, cell_id: CellId) -> Result<(), WfcError> {
    // 获取所有方向的邻居（得益于边对的存在）
    for direction in Direction4::all_directions() {
        if let Some(neighbor) = self.grid.get_neighbor_by_direction(cell_id, direction) {
            // 双向传播约束
            self.update_neighbor_possibilities(neighbor, direction.opposite())?;
        }
    }
    Ok(())
}
```

这种设计确保：

- **完整的邻居信息**：WFC算法能访问所有方向的邻居
- **方向感知传播**：约束传播时知道具体的连接方向  
- **零额外开销**：方向信息通过边创建顺序获得，无需额外存储

## 🎯 核心目标

1. **完整功能迁移**: 确保所有WFC算法逻辑正确迁移
2. **架构优化**: 利用Rust特性改进原设计
3. **性能提升**: 优化算法性能和内存使用
4. **类型安全**: 利用Rust类型系统防止运行时错误
5. **易用性**: 提供清晰的API接口

## 📊 C++原代码分析

### 核心组件分解

#### 1. **数据结构** (30% 复杂度)

```cpp
// C++原结构
enum class State { Collapsed, Noncollapsed, conflict };

struct CellwfcData {
    State state = State::Noncollapsed;
    double entropy = 0.0;
    int randNum = 0;
    Tiles possibility;
};

using WFCSystemData = std::unordered_map<CellID, CellwfcData>;
```

#### 2. **WFC核心算法** (40% 复杂度)

- `collapse()`: 最小熵单元坍塌算法
- `propagateEffects()`: 约束传播算法
- `calculateEntropy()`: 香农熵计算
- `chooseTileFromProbabilities()`: 加权随机选择
- `tileIsCompatible()`: 瓷砖兼容性检查

#### 3. **冲突处理系统** (20% 复杂度)

- `resolveConflicts()`: 冲突解决入口
- `resolveConflictsCell()`: 分层回溯解决
- `recoveryPossibility()`: 可能性恢复
- `retrospectiveGetSolution()`: 深度回溯算法

#### 4. **用户接口** (10% 复杂度)

- `initialize()`: 虚函数初始化
- `run()` / `runStep()`: 执行接口
- `preCollapsed()`: 手动预设
- 各种查询和状态检查方法

## 🚀 Rust迁移策略

### 阶段1: 基础数据结构设计

#### 1.1 状态枚举重新设计

```rust
/// WFC单元格状态，对应C++的State枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CellState {
    /// 未坍塌 - 仍有多种瓷砖可能性
    Uncollapsed,
    /// 已坍塌 - 确定了唯一瓷砖
    Collapsed,
    /// 冲突状态 - 无可行瓷砖选择
    Conflict,
}
```

#### 1.2 单元格WFC数据结构

```rust
/// 单元格WFC附加数据，对应C++的CellwfcData
#[derive(Debug, Clone)]
pub struct CellWfcData {
    /// 单元格当前状态
    pub state: CellState,
    /// 香农熵值
    pub entropy: f64,
    /// 随机数
    pub rand_seed: u64,
    /// 可能的瓷砖列表
    pub possibilities: Vec<TileId>,
}
```

#### 1.3 系统状态管理

```rust
/// WFC系统完整状态，对应C++的WFCSystemData
pub type WfcSystemData = HashMap<CellId, CellWfcData>;

/// 系统状态快照，用于回溯
#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    data: WfcSystemData,
    completed_count: usize,
    timestamp: std::time::Instant,
}
```

### 阶段2: 核心算法实现

#### 2.1 WFC管理器主结构

```rust
/// WFC算法管理器，对应C++的WFCManager模板类
/// 
/// 设计要点：
/// - 基于无向连接的网格系统，确保约束传播的双向性
/// - 利用方向识别机制进行精确的约束检查
/// - 集成边对管理，确保WFC算法的完整性
pub struct WfcManager<EdgeData>
where
    EdgeData: Clone + PartialEq + std::fmt::Debug + Send + Sync,
{
    /// 网格系统引用（基于无向连接设计）
    grid: GridSystem,
    /// 瓷砖集引用  
    tile_set: Box<dyn TileSetVirtual<EdgeData>>,
    /// WFC系统数据
    wfc_data: WfcSystemData,
    /// 已完成单元计数
    completed_count: usize,
    /// 随机数生成器
    rng: StdRng,
    /// 配置参数
    config: WfcConfig,
}
```

#### 2.2 配置参数结构

```rust
/// WFC算法配置参数
#[derive(Debug, Clone)]
pub struct WfcConfig {
    /// 最大递归深度
    pub max_recursion_depth: usize,
    /// 随机种子
    pub random_seed: Option<u64>,

}
```

#### 2.3 核心算法方法设计

##### 坍塌算法

```rust
impl<EdgeData> WfcManager<EdgeData> {
    /// 主坍塌算法，对应C++的collapse()
    fn collapse(&mut self) -> Result<(), WfcError> {
        // 1. 找到最小熵单元
        let min_entropy_cell = self.find_min_entropy_cell()?;
        
        // 2. 从概率分布中选择瓷砖
        let chosen_tile = self.choose_tile_from_probabilities(min_entropy_cell)?;
        
        // 3. 设置瓷砖并更新状态
        self.set_tile_for_cell(min_entropy_cell, chosen_tile)?;
        
        // 4. 传播约束效果
        self.propagate_effects(min_entropy_cell)?;
        
        Ok(())
    }
    
    /// 寻找最小熵单元格
    fn find_min_entropy_cell(&self) -> Result<CellId, WfcError> {
        self.wfc_data
            .iter()
            .filter(|(_, data)| data.state == CellState::Uncollapsed)
            .min_by(|(_, a), (_, b)| a.entropy.partial_cmp(&b.entropy).unwrap())
            .map(|(&cell_id, _)| cell_id)
            .ok_or(WfcError::NoUncollapsedCells)
    }
}
```

##### 约束传播算法

```rust
impl<EdgeData> WfcManager<EdgeData> {
    /// 约束传播算法，对应C++的propagateEffects()
    /// 
    /// 利用无向连接（边对）进行双向约束传播，确保所有邻居的约束一致性
    fn propagate_effects(&mut self, start_cell: CellId) -> Result<(), WfcError> {
        let mut propagation_queue = VecDeque::new();
        let mut processed_cells = HashSet::new();
        
        propagation_queue.push_back(start_cell);
        processed_cells.insert(start_cell);
        
        while let Some(current_cell) = propagation_queue.pop_front() {
            // 获取所有方向的邻居（利用边对的双向可达性）
            for direction in Direction4::all_directions() {
                if let Some(neighbor) = self.grid.get_neighbor_by_direction(current_cell, direction) {
                    if processed_cells.contains(&neighbor) {
                        continue;
                    }
                    
                    // 更新邻居可能性，传入明确的连接方向
                    let constraint_updated = self.update_neighbor_possibilities(
                        neighbor, 
                        current_cell,
                        direction.opposite().unwrap() // 从邻居看向当前单元的方向
                    )?;
                    
                    if constraint_updated {
                        propagation_queue.push_back(neighbor);
                        processed_cells.insert(neighbor);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// 更新邻居可能性，基于方向感知的约束检查
    fn update_neighbor_possibilities(
        &mut self, 
        neighbor: CellId, 
        source: CellId, 
        connection_direction: Direction4
    ) -> Result<bool, WfcError> {
        let neighbor_data = self.wfc_data.get_mut(&neighbor)
            .ok_or(WfcError::CellNotFound(neighbor))?;
            
        if neighbor_data.state != CellState::Uncollapsed {
            return Ok(false); // 已坍塌或冲突的单元格不需要更新
        }
        
        let source_possibilities = &self.wfc_data[&source].possibilities;
        let mut updated = false;
        
        // 过滤掉与源单元格不兼容的瓷砖
        neighbor_data.possibilities.retain(|&tile_id| {
            source_possibilities.iter().any(|&source_tile| {
                self.tile_set.judge_possibility(
                    &[vec![source_tile]], // 邻居可能性
                    tile_id
                )
            })
        });
        
        // 检查是否产生了约束变化
        if neighbor_data.possibilities.len() != neighbor_data.possibilities.len() {
            updated = true;
            
            // 重新计算熵值
            neighbor_data.entropy = self.calculate_entropy(&neighbor_data.possibilities);
            
            // 检查冲突状态
            if neighbor_data.possibilities.is_empty() {
                neighbor_data.state = CellState::Conflict;
            }
        }
        
        Ok(updated)
    }
}
```

##### 熵计算算法

```rust
impl<EdgeData> WfcManager<EdgeData> {
    /// 计算香农熵，对应C++的calculateEntropy()
    fn calculate_entropy(&self, possibilities: &[TileId]) -> f64 {
        if possibilities.is_empty() {
            return 0.0;
        }
        
        if possibilities.len() == 1 {
            return 0.0;
        }
        
        // 计算总权重
        let total_weight: f64 = possibilities
            .iter()
            .filter_map(|&tile_id| self.tile_set.get_tile(tile_id))
            .map(|tile| tile.weight as f64)
            .sum();
            
        if total_weight == 0.0 {
            return (possibilities.len() as f64).log2();
        }
        
        // 计算香农熵
        possibilities
            .iter()
            .filter_map(|&tile_id| self.tile_set.get_tile(tile_id))
            .map(|tile| tile.weight as f64 / total_weight)
            .filter(|&prob| prob > 0.0)
            .map(|prob| -prob * prob.log2())
            .sum()
    }
}
```

### 阶段3: 冲突处理系统

#### 3.1 分层冲突修复机制

**重要概念澄清**：本系统的冲突处理使用**分层修复方法**，这里的"回溯"是专门为解决冲突层而设计的，**不同于传统WFC算法的过程性回溯**。

**分层修复的特点**：

1. **冲突定位**：识别并收集所有冲突单元格
2. **分层扩展**：从冲突核心向外扩展影响层
3. **逐层修复**：从外层到内层恢复可能性
4. **局部回溯**：在修复过程中使用回溯算法寻找可行解

```rust
/// 冲突解决系统
impl<EdgeData> WfcManager<EdgeData> {
    /// 解决所有冲突，使用统一的分层修复方法
    pub fn resolve_conflicts(&mut self) -> Result<bool, WfcError> {
        let conflict_cells = self.collect_conflict_cells();
        
        if conflict_cells.is_empty() {
            return Ok(true);
        }
        
        // 使用分层回溯解决所有冲突
        self.layered_backtrack_resolution(conflict_cells)
    }
    
    /// 分层回溯解决，对应C++的resolveConflictsCell()
    /// 
    /// 这是WFC系统的核心冲突修复机制，通过分层回溯来解决冲突。
    /// 不同于传统WFC的过程性回溯，这里的回溯是专门为解决冲突层而设计的。
    fn layered_backtrack_resolution(&mut self, conflict_cells: Vec<CellId>) -> Result<bool, WfcError> {
        let mut layers = vec![conflict_cells];
        self.resolve_conflicts_recursive(&mut layers, 0)
    }
    
    /// 递归冲突解决 - 分层修复的核心算法
    /// 
    /// 这个方法实现了分层冲突修复的递归逻辑，通过逐层扩展和回溯来解决冲突。
    /// 注意：这里的"回溯"是为冲突修复设计的，不同于传统WFC算法的过程性回溯。
    fn resolve_conflicts_recursive(
        &mut self, 
        layers: &mut Vec<Vec<CellId>>, 
        depth: usize
    ) -> Result<bool, WfcError> {
        if depth >= self.config.max_recursion_depth {
            return Ok(false);
        }
        
        // 从外层到内层解决冲突
        for layer_idx in (0..layers.len()).rev() {
            for &cell in &layers[layer_idx].clone() {
                self.recover_cell_possibilities(cell, layers)?;
            }
        }
        
        // 尝试获取解决方案
        let all_cells: Vec<CellId> = layers.iter().flatten().copied().collect();
        if self.backtrack_solution(&all_cells, 0)? {
            return Ok(true);
        }
        
        // 如果失败，扩展到下一层
        if depth < self.config.max_recursion_depth - 1 {
            let next_layer = self.build_next_layer(&layers[depth])?;
            if !next_layer.is_empty() {
                layers.push(next_layer);
                return self.resolve_conflicts_recursive(layers, depth + 1);
            }
        }
        
        Ok(false)
    }
}
```

#### 3.2 分层修复中的回溯算法

**特别说明**：以下回溯算法是专门为分层冲突修复设计的，用于在修复过程中寻找局部可行解，不是传统WFC的全局回溯。

```rust
impl<EdgeData> WfcManager<EdgeData> {
    /// 回溯求解算法，对应C++的retrospectiveGetSolution()
    /// 
    /// 这是分层修复过程中使用的局部回溯算法，用于在冲突修复时寻找可行的瓷砖组合。
    /// 注意：这不是传统WFC的全局回溯，而是针对特定冲突层的局部求解。
    fn backtrack_solution(&mut self, cells: &[CellId], index: usize) -> Result<bool, WfcError> {
        if index >= cells.len() {
            return Ok(true);
        }
        
        let cell = cells[index];
        let cell_data = self.wfc_data.get(&cell).ok_or(WfcError::CellNotFound)?;
        
        if cell_data.possibilities.is_empty() {
            return Ok(false);
        }
        
        // 保存当前状态
        let snapshot = self.create_snapshot();
        
        // 尝试每种可能性
        for &possibility in &cell_data.possibilities.clone() {
            if self.is_tile_compatible(possibility, cell)? {
                // 设置瓷砖
                self.set_tile_for_cell(cell, possibility)?;
                
                // 递归处理下一个单元
                if self.backtrack_solution(cells, index + 1)? {
                    return Ok(true);
                }
                
                // 恢复状态
                self.restore_snapshot(snapshot.clone())?;
            }
        }
        
        Ok(false)
    }
    
    /// 创建系统快照
    fn create_snapshot(&self) -> SystemSnapshot {
        SystemSnapshot {
            data: self.wfc_data.clone(),
            completed_count: self.completed_count,
            timestamp: std::time::Instant::now(),
        }
    }
    
    /// 恢复系统快照
    fn restore_snapshot(&mut self, snapshot: SystemSnapshot) -> Result<(), WfcError> {
        self.wfc_data = snapshot.data;
        self.completed_count = snapshot.completed_count;
        Ok(())
    }
}
```

### 阶段4: 用户接口设计

#### 4.1 初始化接口

```rust
/// 初始化特性，对应C++的initialize()虚函数
pub trait WfcInitializer<EdgeData> {
    /// 初始化WFC系统
    fn initialize(&mut self, manager: &mut WfcManager<EdgeData>) -> Result<(), WfcError>;
}

/// 默认初始化器
pub struct DefaultInitializer;

impl<EdgeData> WfcInitializer<EdgeData> for DefaultInitializer 
where
    EdgeData: Clone + PartialEq + std::fmt::Debug + Send + Sync,
{
    fn initialize(&mut self, manager: &mut WfcManager<EdgeData>) -> Result<(), WfcError> {
        // 1. 构建瓷砖集
        manager.tile_set.build_tile_set();
        
        // 2. 初始化所有单元格
        for cell_id in manager.grid.get_all_cells() {
            let cell_data = CellWfcData {
                state: CellState::Uncollapsed,
                entropy: 0.0,
                rand_seed: manager.rng.gen(),
                possibilities: manager.tile_set.get_all_tile_ids(),
            };
            manager.wfc_data.insert(cell_id, cell_data);
        }
        
        // 3. 计算初始熵值
        manager.update_all_entropies()?;
        
        Ok(())
    }
}
```

#### 4.2 执行接口

```rust
impl<EdgeData> WfcManager<EdgeData> {
    /// 完整运行WFC算法，对应C++的run()
    pub fn run(&mut self) -> Result<(), WfcError> {
        while !self.is_complete() {
            self.collapse()?;
        }
        
        // 解决剩余冲突
        if !self.resolve_conflicts()? {
            return Err(WfcError::UnresolvableConflicts);
        }
        
        Ok(())
    }
    
    /// 单步执行，对应C++的runStep()
    pub fn run_step(&mut self) -> Result<StepResult, WfcError> {
        if self.is_complete() {
            if self.has_conflicts() {
                if self.resolve_conflicts()? {
                    Ok(StepResult::ConflictsResolved)
                } else {
                    Ok(StepResult::ConflictResolutionFailed)
                }
            } else {
                Ok(StepResult::Complete)
            }
        } else {
            self.collapse()?;
            Ok(StepResult::Collapsed)
        }
    }
    
    /// 预设单元格，对应C++的preCollapsed()
    pub fn pre_collapse(&mut self, cell: CellId, tile: TileId) -> Result<(), WfcError> {
        let cell_data = self.wfc_data.get_mut(&cell).ok_or(WfcError::CellNotFound)?;
        
        if cell_data.state != CellState::Uncollapsed {
            return Err(WfcError::CellAlreadyCollapsed);
        }
        
        if !cell_data.possibilities.contains(&tile) {
            return Err(WfcError::InvalidTileChoice);
        }
        
        self.set_tile_for_cell(cell, tile)?;
        self.propagate_effects(cell)?;
        
        Ok(())
    }
    
    /// 检查是否完成
    pub fn is_complete(&self) -> bool {
        self.completed_count == self.grid.get_cells_count()
    }
}
```

#### 4.3 执行结果类型

```rust
/// 单步执行结果
#[derive(Debug, Clone, PartialEq)]
pub enum StepResult {
    /// 成功坍塌一个单元
    Collapsed,
    /// 解决了冲突
    ConflictsResolved,
    /// 冲突解决失败
    ConflictResolutionFailed,
    /// 完成
    Complete,
}
```

### 阶段5: 错误处理系统

#### 5.1 WFC特定错误类型

```rust
/// WFC算法特定错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum WfcError {
    /// 网格系统错误
    Grid(GridError),
    /// 没有未坍塌的单元格
    NoUncollapsedCells,
    /// 单元格未找到
    CellNotFound(CellId),
    /// 瓷砖未找到
    TileNotFound,
    /// 单元格已坍塌
    CellAlreadyCollapsed,
    /// 无效的瓷砖选择
    InvalidTileChoice,
    /// 无法解决的冲突
    UnresolvableConflicts,
    /// 系统状态不一致
    InconsistentState,
    /// 初始化失败
    InitializationFailed(String),
}

impl From<GridError> for WfcError {
    fn from(error: GridError) -> Self {
        WfcError::Grid(error)
    }
}

impl std::fmt::Display for WfcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WfcError::Grid(e) => write!(f, "Grid error: {}", e),
            WfcError::NoUncollapsedCells => write!(f, "No uncollapsed cells available"),
            WfcError::CellNotFound(cell_id) => write!(f, "Cell not found in WFC data: {}", cell_id),
            WfcError::TileNotFound => write!(f, "Tile not found in tile set"),
            WfcError::CellAlreadyCollapsed => write!(f, "Cell is already collapsed"),
            WfcError::InvalidTileChoice => write!(f, "Invalid tile choice for cell"),
            WfcError::UnresolvableConflicts => write!(f, "Conflicts cannot be resolved"),
            WfcError::InconsistentState => write!(f, "WFC system state is inconsistent"),
            WfcError::InitializationFailed(msg) => write!(f, "Initialization failed: {}", msg),
        }
    }
}

impl std::error::Error for WfcError {}
```

## 📅 实施计划

### 第1周: 基础架构 (阶段1)

- [ ] 创建`wfc_manager.rs`模块
- [ ] 实现基础数据结构
- [ ] 设计错误处理系统
- [ ] 建立单元测试框架

### 第2周: 核心算法 (阶段2)

- [ ] 实现`WfcManager`主结构
- [ ] 迁移坍塌算法
- [ ] 实现约束传播
- [ ] 添加熵计算功能

### 第3周: 冲突处理 (阶段3)

- [ ] 实现冲突检测
- [ ] 迁移回溯算法
- [ ] 添加分层解决机制
- [ ] 性能优化

### 第4周: 用户接口 (阶段4+5)

- [ ] 设计初始化系统
- [ ] 实现执行接口
- [ ] 完善错误处理
- [ ] 编写文档和示例

## 🧪 测试策略

### 单元测试

- 每个算法组件的独立测试
- 边界条件和错误情况测试
- 性能基准测试

### 集成测试

- 完整WFC流程测试
- 与现有模块的集成测试
- 不同网格类型的兼容性测试

### 性能测试

- 大规模网格的性能测试
- 内存使用优化验证
- 与C++版本的性能对比

## 🎯 成功标准

1. **功能完整性**: 所有C++功能正确迁移
2. **性能表现**: 不低于原C++版本性能
3. **代码质量**: 通过所有测试，无内存安全问题
4. **API友好性**: 提供清晰易用的Rust API
5. **文档完整**: 完整的API文档和使用示例

## 🔧 技术要点

### 关键优化机会

1. **内存管理**: 利用Rust所有权系统避免不必要的复制
2. **并发安全**: 设计支持并发访问的数据结构
3. **缓存优化**: 实现熵值和兼容性检查的缓存
4. **算法改进**: 优化回溯算法的性能

### 潜在挑战

1. **复杂度管理**: C++代码的复杂逻辑需要仔细重构
2. **性能要求**: 保证迁移后的性能不下降
3. **API设计**: 在保持功能完整性的同时提供易用接口
4. **测试覆盖**: 确保所有边界情况都得到测试

### 已解决的设计问题 ✅

- **图结构选择**: 确定使用petgraph有向图实现无向连接的方向识别
- **无向连接保证**: 通过边对确保WFC算法需要的双向连通性
- **方向识别机制**: 基于边创建顺序和petgraph特性的零成本方向识别
- **约束传播优化**: 方向感知的约束传播，提高算法精确性
- **错误类型**: 完整定义了WfcError及其错误处理
- **内存安全**: 所有数据结构都使用Rust的安全抽象
- **并发支持**: 设计支持未来的并行WFC实现
- **状态管理**: 清晰的状态转换和快照机制

### 3. 瓷砖管理系统

#### 核心数据结构设计

##### 瓷砖数据结构

```rust
pub struct Tile<EdgeData> {
    pub id: TileId,
    pub weight: i32,
    pub edges: Vec<EdgeData>,  // 🎯 边数据顺序至关重要！
}
```

###### **⚠️ 关键约束：瓷砖边数据顺序**

瓷砖的边数据必须严格按照 `neighbors()` 返回顺序排列：

```rust
// ✅ 正确的瓷砖边数据顺序
let tile_edges = vec![
    "北边数据",  // 索引 0 - 对应网格中北方向邻居
    "西边数据",  // 索引 1 - 对应网格中西方向邻居  
    "南边数据",  // 索引 2 - 对应网格中南方向邻居
    "东边数据",  // 索引 3 - 对应网格中东方向邻居
];
tile_set.add_tile(tile_edges, weight);
```

**顺序对应关系**：

```text
网格边创建顺序：东 → 南 → 西 → 北
petgraph.neighbors()：[北, 西, 南, 东] (逆序返回)
瓷砖边数据索引：  [0,  1,  2,  3]
方向到索引映射：  北=0, 西=1, 南=2, 东=3
```

**设计优势**：

1. **直接索引对应**：无需额外映射转换
2. **高效兼容性检查**：O(1)时间获取对应边数据
3. **统一的顺序约定**：网格和瓷砖使用相同的索引语义

#### 兼容性判断算法优化

利用顺序对应关系，可以实现高效的边兼容性检查：

```rust
impl TileSetVirtual<EdgeData> for MyTileSet {
    fn judge_possibility(
        &self,
        neighbor_possibilities: &[Vec<TileId>],
        candidate: TileId
    ) -> bool {
        let candidate_tile = self.get_tile(candidate).unwrap();
        
        // 遍历每个方向的邻居约束
        for (direction_index, neighbor_tiles) in neighbor_possibilities.iter().enumerate() {
            if neighbor_tiles.is_empty() { continue; }
            
            // 🎯 直接通过索引获取候选瓷砖的边数据
            let candidate_edge = &candidate_tile.edges[direction_index];
            
            // 检查是否与任意邻居瓷砖兼容
            let is_compatible = neighbor_tiles.iter().any(|&neighbor_id| {
                if let Some(neighbor_tile) = self.get_tile(neighbor_id) {
                    // 获取邻居瓷砖相对方向的边数据
                    let opposite_index = Self::get_opposite_direction_index(direction_index);
                    let neighbor_edge = &neighbor_tile.edges[opposite_index];
                    
                    // 边兼容性检查（具体规则由应用定义）
                    candidate_edge == neighbor_edge  // 或其他兼容性规则
                } else {
                    false
                }
            });
            
            if !is_compatible {
                return false;  // 该方向不兼容
            }
        }
        
        true  // 所有方向都兼容
    }
}

impl MyTileSet {
    /// 获取相对方向的索引
    fn get_opposite_direction_index(direction_index: usize) -> usize {
        match direction_index {
            0 => 2,  // 北 ↔ 南
            1 => 3,  // 西 ↔ 东  
            2 => 0,  // 南 ↔ 北
            3 => 1,  // 东 ↔ 西
            _ => direction_index,  // 错误情况的回退
        }
    }
}
```
