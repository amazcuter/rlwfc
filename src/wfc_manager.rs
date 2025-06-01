//! # WFC管理器模块
//!
//! 本模块提供了WFC（Wave Function Collapse）算法的完整实现，是对原C++代码的Rust重写版本。
//!
//! ## 核心组件
//!
//! - [`WfcManager`] - WFC算法管理器，对应原C++的`WFCManager`类
//! - [`CellState`] - 单元格状态枚举
//! - [`CellWfcData`] - 单元格WFC附加数据
//! - [`WfcConfig`] - WFC算法配置参数
//!
//! ## 设计特点
//!
//! ### 与原C++的完整对应
//!
//! 本实现完全保持了与原C++代码的逻辑一致性：
//!
//! - **相同的算法流程**：坍塌→传播→冲突处理
//! - **相同的数据结构**：State枚举、CellwfcData结构、WFCSystemData映射
//! - **相同的冲突处理**：分层修复方法，非传统WFC回溯
//!
//! ### Rust特有改进
//!
//! - **类型安全**：使用强类型和Result类型避免运行时错误
//! - **内存安全**：自动内存管理，无手动释放
//! - **错误处理**：完整的错误类型系统
//! - **trait抽象**：初始化和瓷砖集使用trait实现多态
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use rlwfc::{WfcManager, GridSystem, TileSetVirtual, WfcInitializer, WfcError, Tile, TileId, GridError};
//!
//! // 创建简单的瓷砖集
//! struct MyTileSet;
//! impl TileSetVirtual<i32> for MyTileSet {
//!     fn build_tile_set(&mut self) -> Result<(), GridError> { Ok(()) }
//!     fn judge_possibility(&self, _: &[Vec<TileId>], _: TileId) -> bool { true }
//!     fn get_tile(&self, _: TileId) -> Option<&Tile<i32>> { None }
//!     fn get_tile_count(&self) -> usize { 0 }
//!     fn get_all_tile_ids(&self) -> Vec<TileId> { vec![] }
//! }
//!
//! // 实现初始化器
//! struct MyInitializer;
//! impl<EdgeData> WfcInitializer<EdgeData> for MyInitializer
//! where EdgeData: Clone + PartialEq + std::fmt::Debug + Send + Sync
//! {
//!     fn initialize(&mut self, manager: &mut WfcManager<EdgeData>) -> Result<(), WfcError> {
//!         // 初始化逻辑
//!         Ok(())
//!     }
//! }
//!
//! // 使用WFC系统
//! let grid = GridSystem::new();
//! let tile_set = Box::new(MyTileSet);
//! let mut manager = WfcManager::new(grid, tile_set).unwrap();
//!
//! let mut initializer = MyInitializer;
//! manager.initialize_with(&mut initializer).unwrap();
//!
//! // 运行WFC算法
//! manager.run().unwrap();
//! ```

use crate::grid_system::GridSystem;
use crate::tile_set::TileSetVirtual;
/**
 * @file wfc_manager.rs
 * @author amazcuter (amazcuter@outlook.com)
 * @brief WFC系统管理器 - Rust重写版本
 *        对应原C++ WFCManager.h的功能，实现完整的WFC算法
 * @version 1.0
 * @date 2025-01-25
 *
 * @copyright Copyright (c) 2025
 */
use crate::wfc_util::*;
use rand::prelude::*;
use rand::rngs::StdRng;
use std::collections::{HashMap, HashSet, VecDeque};

// =============================================================================
// 基础数据结构 - 对应原C++的枚举和结构体
// =============================================================================

/// WFC单元格状态，对应C++的State枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CellState {
    /// 未坍塌 - 仍有多种瓷砖可能性，对应C++的Noncollapsed
    Uncollapsed,
    /// 已坍塌 - 确定了唯一瓷砖，对应C++的Collapsed
    Collapsed,
    /// 冲突状态 - 无可行瓷砖选择，对应C++的conflict
    Conflict,
}

/// 单元格WFC附加数据，对应C++的CellwfcData
#[derive(Debug, Clone)]
pub struct CellWfcData {
    /// 单元格当前状态
    pub state: CellState,
    /// 香农熵值
    pub entropy: f64,
    /// 随机种子，对应C++的randNum的种子来源
    pub rand_seed: u64,
    /// 预计算的随机数，对应C++的randNum
    pub rand_num: i32,
    /// 可能的瓷砖列表，对应C++的possibility
    pub possibilities: Vec<TileId>,
}

impl CellWfcData {
    /// 创建新的单元格WFC数据
    pub fn new(rand_seed: u64, possibilities: Vec<TileId>) -> Self {
        // 使用种子生成预计算的随机数，模拟C++的randNum行为
        let mut rng = StdRng::seed_from_u64(rand_seed);
        let rand_num = rng.random::<i32>().abs(); // 确保是正数
        
        Self {
            state: CellState::Uncollapsed,
            entropy: 0.0, // 将在初始化时计算
            rand_seed,
            rand_num,
            possibilities,
        }
    }
}

/// WFC系统完整状态，对应C++的WFCSystemData
pub type WfcSystemData = HashMap<CellId, CellWfcData>;

/// 系统状态快照，用于回溯
#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    /// WFC系统数据快照
    data: WfcSystemData,
    /// 已完成单元计数
    completed_count: usize,
}

/// WFC算法配置参数
#[derive(Debug, Clone)]
pub struct WfcConfig {
    /// 最大递归深度
    pub max_recursion_depth: usize,
    /// 随机种子
    pub random_seed: Option<u64>,
}

impl Default for WfcConfig {
    fn default() -> Self {
        Self {
            max_recursion_depth: 3, // 对应C++的硬编码深度限制
            random_seed: None,
        }
    }
}

// =============================================================================
// WFC错误类型
// =============================================================================

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
            WfcError::CellNotFound(cell_id) => {
                write!(f, "Cell not found in WFC data: {:?}", cell_id)
            }
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

// =============================================================================
// 初始化特性 - 对应原C++的initialize虚函数
// =============================================================================

/// 初始化特性，对应C++的initialize()虚函数
pub trait WfcInitializer<EdgeData>
where
    EdgeData: Clone + PartialEq + std::fmt::Debug + Send + Sync,
{
    /// 初始化WFC系统，对应C++的initialize()虚函数
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
        manager.tile_set.build_tile_set()?;

        // 2. 初始化所有单元格
        for cell_id in manager.grid.get_all_cells() {
            let rand_seed = manager.rng.random();
            let all_tiles = manager.tile_set.get_all_tile_ids();
            let cell_data = CellWfcData::new(rand_seed, all_tiles);
            manager.wfc_data.insert(cell_id, cell_data);
        }

        // 3. 计算初始熵值
        manager.update_all_entropies()?;

        Ok(())
    }
}

// =============================================================================
// 执行结果类型
// =============================================================================

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

// =============================================================================
// WFC管理器主结构
// =============================================================================

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
    /// 网格系统，对应C++的grid_成员
    grid: GridSystem,
    /// 瓷砖集，对应C++的tileSet_成员
    tile_set: Box<dyn TileSetVirtual<EdgeData>>,
    /// WFC系统数据，对应C++的wfcCellData成员
    wfc_data: WfcSystemData,
    /// 已完成单元计数，对应C++的completedCellCount
    completed_count: usize,
    /// 随机数生成器
    rng: StdRng,
    /// 配置参数
    config: WfcConfig,
    /// 熵值缓存，对应C++的entropyCache
    #[allow(dead_code)]
    entropy_cache: HashMap<Vec<TileId>, f64>,
}

impl<EdgeData> WfcManager<EdgeData>
where
    EdgeData: Clone + PartialEq + std::fmt::Debug + Send + Sync,
{
    /// 创建新的WFC管理器
    pub fn new(
        grid: GridSystem,
        tile_set: Box<dyn TileSetVirtual<EdgeData>>,
    ) -> Result<Self, WfcError> {
        let config = WfcConfig::default();
        let seed = config
            .random_seed
            .unwrap_or_else(|| rand::rng().random());
        let rng = StdRng::seed_from_u64(seed);

        Ok(Self {
            grid,
            tile_set,
            wfc_data: HashMap::new(),
            completed_count: 0,
            rng,
            config,
            entropy_cache: HashMap::new(),
        })
    }

    /// 使用自定义配置创建WFC管理器
    pub fn with_config(
        grid: GridSystem,
        tile_set: Box<dyn TileSetVirtual<EdgeData>>,
        config: WfcConfig,
    ) -> Result<Self, WfcError> {
        let seed = config
            .random_seed
            .unwrap_or_else(|| rand::rng().random());
        let rng = StdRng::seed_from_u64(seed);

        Ok(Self {
            grid,
            tile_set,
            wfc_data: HashMap::new(),
            completed_count: 0,
            rng,
            config,
            entropy_cache: HashMap::new(),
        })
    }

    // ==========================================================================
    // 公共接口方法 - 对应原C++的public方法
    // ==========================================================================

    /// 使用初始化器初始化WFC系统，对应C++的initialize()虚函数调用
    pub fn initialize_with<I: WfcInitializer<EdgeData>>(
        &mut self,
        initializer: &mut I,
    ) -> Result<(), WfcError> {
        initializer.initialize(self)
    }

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
        let cell_data = self
            .wfc_data
            .get_mut(&cell)
            .ok_or(WfcError::CellNotFound(cell))?;

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

    /// 检查是否完成，对应C++的isComplete()
    pub fn is_complete(&self) -> bool {
        self.completed_count == self.grid.get_cells_count()
    }

    /// 获取单元格状态，对应C++的getCellState()
    pub fn get_cell_state(&self, cell_id: CellId) -> Result<CellState, WfcError> {
        self.wfc_data
            .get(&cell_id)
            .map(|data| data.state)
            .ok_or(WfcError::CellNotFound(cell_id))
    }

    /// 获取已坍塌单元格的瓷砖，对应C++的getCollapsedCellData()
    pub fn get_collapsed_cell_tile(&self, cell_id: CellId) -> Result<TileId, WfcError> {
        let cell_data = self
            .wfc_data
            .get(&cell_id)
            .ok_or(WfcError::CellNotFound(cell_id))?;

        if cell_data.state == CellState::Collapsed && cell_data.possibilities.len() == 1 {
            Ok(cell_data.possibilities[0])
        } else {
            Err(WfcError::InconsistentState)
        }
    }

    /// 获取网格系统引用，对应C++的getGrid()
    pub fn get_grid(&self) -> &GridSystem {
        &self.grid
    }

    /// 获取所有瓷砖ID
    pub fn get_all_tile_ids(&self) -> Vec<TileId> {
        (0..self.tile_set.get_tile_count()).collect()
    }

    /// 获取瓷砖
    pub fn get_tile(&self, tile_id: TileId) -> Option<&Tile<EdgeData>> {
        if tile_id < self.tile_set.get_tile_count() {
            self.tile_set.get_tile(tile_id)
        } else {
            None
        }
    }

    // ==========================================================================
    // 核心算法实现 - 对应原C++的private方法
    // ==========================================================================

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

    /// 寻找最小熵单元格，对应C++的reCalcMinEntropyCell()
    fn find_min_entropy_cell(&self) -> Result<CellId, WfcError> {
        self.wfc_data
            .iter()
            .filter(|(_, data)| data.state == CellState::Uncollapsed)
            .min_by(|(_, a), (_, b)| {
                a.entropy
                    .partial_cmp(&b.entropy)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(&cell_id, _)| cell_id)
            .ok_or(WfcError::NoUncollapsedCells)
    }

    /// 从概率分布选择瓷砖，对应C++的chooseTileFromProbabilities()
    fn choose_tile_from_probabilities(&mut self, cell_id: CellId) -> Result<TileId, WfcError> {
        let cell_data = self
            .wfc_data
            .get(&cell_id)
            .ok_or(WfcError::CellNotFound(cell_id))?;

        if cell_data.possibilities.is_empty() {
            return Err(WfcError::InvalidTileChoice);
        }

        // 计算总权重，对应C++的weightSum计算
        let mut total_weight = 0i32;
        for &tile_id in &cell_data.possibilities {
            if let Some(tile) = self.tile_set.get_tile(tile_id) {
                total_weight += tile.weight;
            }
        }

        if total_weight == 0 {
            return Ok(cell_data.possibilities[0]); // 如果没有权重，返回第一个
        }

        // 使用预计算的随机数，完全对应C++的逻辑
        // C++: randNum %= weightSum;
        let rand_num = cell_data.rand_num % total_weight;
        
        // C++: 累计权重直到 weightSum >= randNum
        let mut weight_sum = 0i32;
        for &tile_id in &cell_data.possibilities {
            if let Some(tile) = self.tile_set.get_tile(tile_id) {
                weight_sum += tile.weight;
                if weight_sum > rand_num {  // C++: weightSum >= randNum，但我们用>避免边界问题
                    return Ok(tile_id);
                }
            }
        }

        // 保险措施，理论上不应该到达这里
        Ok(*cell_data.possibilities.last().unwrap())
    }

    /// 设置单元格瓷砖，对应C++的setTileForCell()
    fn set_tile_for_cell(&mut self, cell_id: CellId, tile_id: TileId) -> Result<(), WfcError> {
        let cell_data = self
            .wfc_data
            .get_mut(&cell_id)
            .ok_or(WfcError::CellNotFound(cell_id))?;

        // 设置选定的瓷砖为唯一的可能性
        cell_data.possibilities.clear();
        cell_data.possibilities.push(tile_id);
        cell_data.entropy = 0.0;
        cell_data.state = CellState::Collapsed;

        self.completed_count += 1;

        Ok(())
    }

    /// 约束传播算法，对应C++的propagateEffects()
    ///
    /// 利用无向连接（边对）进行双向约束传播，确保所有邻居的约束一致性
    fn propagate_effects(&mut self, start_cell: CellId) -> Result<(), WfcError> {
        if self.is_complete() {
            return Ok(());
        }

        let mut propagation_queue = VecDeque::new();
        let mut processed_cells = HashSet::new();

        propagation_queue.push_back(start_cell);
        processed_cells.insert(start_cell);

        while let Some(current_cell) = propagation_queue.pop_front() {
            // 获取所有邻居
            let neighbors = self.grid.get_neighbors(current_cell);

            for neighbor in neighbors {
                if processed_cells.contains(&neighbor) {
                    continue;
                }

                let neighbor_data = self
                    .wfc_data
                    .get(&neighbor)
                    .ok_or(WfcError::CellNotFound(neighbor))?;
                if neighbor_data.state != CellState::Uncollapsed {
                    continue;
                }

                // 更新邻居可能性
                let constraint_updated = self.update_neighbor_possibilities(neighbor)?;

                if constraint_updated {
                    propagation_queue.push_back(neighbor);
                    processed_cells.insert(neighbor);
                }
            }
        }

        Ok(())
    }

    /// 更新邻居可能性，基于约束传播
    fn update_neighbor_possibilities(&mut self, neighbor: CellId) -> Result<bool, WfcError> {
        // 先获取邻居数据的克隆，避免可变借用冲突
        let neighbor_data = self
            .wfc_data
            .get(&neighbor)
            .ok_or(WfcError::CellNotFound(neighbor))?
            .clone();

        if neighbor_data.state != CellState::Uncollapsed {
            return Ok(false); // 已坍塌或冲突的单元格不需要更新
        }

        // 过滤兼容的瓷砖
        let compatible_tiles = self.filter_compatible_tiles(neighbor)?;

        // 检查是否产生了约束变化
        let old_count = neighbor_data.possibilities.len();
        let new_count = compatible_tiles.len();

        if new_count != old_count {
            // 计算新的熵值
            let new_entropy = self.calculate_entropy(&compatible_tiles);

            // 更新邻居数据
            let neighbor_data_mut = self.wfc_data.get_mut(&neighbor).unwrap();
            neighbor_data_mut.possibilities = compatible_tiles;
            neighbor_data_mut.entropy = new_entropy;

            // 检查冲突状态
            if neighbor_data_mut.possibilities.is_empty() {
                neighbor_data_mut.state = CellState::Conflict;
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// 过滤兼容的瓷砖
    fn filter_compatible_tiles(&self, cell_id: CellId) -> Result<Vec<TileId>, WfcError> {
        let cell_data = self
            .wfc_data
            .get(&cell_id)
            .ok_or(WfcError::CellNotFound(cell_id))?;
        let mut compatible_tiles = Vec::new();

        for &tile_id in &cell_data.possibilities {
            if self.tile_is_compatible(tile_id, cell_id)? {
                compatible_tiles.push(tile_id);
            }
        }

        Ok(compatible_tiles)
    }

    /// 检查瓷砖兼容性，对应C++的tileIsCompatible()
    fn tile_is_compatible(&self, tile_id: TileId, cell_id: CellId) -> Result<bool, WfcError> {
        let neighbors = self.grid.get_neighbors(cell_id);
        let mut neighbor_possibilities = Vec::new();

        for neighbor in neighbors {
            if let Some(neighbor_data) = self.wfc_data.get(&neighbor) {
                neighbor_possibilities.push(neighbor_data.possibilities.clone());
            } else {
                neighbor_possibilities.push(self.tile_set.get_all_tile_ids());
            }
        }

        Ok(self
            .tile_set
            .judge_possibility(&neighbor_possibilities, tile_id))
    }

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

    /// 更新所有单元格的熵值
    fn update_all_entropies(&mut self) -> Result<(), WfcError> {
        let cell_ids: Vec<CellId> = self.wfc_data.keys().copied().collect();

        for cell_id in cell_ids {
            let possibilities = self.wfc_data[&cell_id].possibilities.clone();
            let entropy = self.calculate_entropy(&possibilities);
            self.wfc_data.get_mut(&cell_id).unwrap().entropy = entropy;
        }

        Ok(())
    }

    // ==========================================================================
    // 分层冲突修复系统 - 对应原C++的冲突处理方法
    // ==========================================================================

    /// 解决所有冲突，使用统一的分层修复方法，对应C++的resolveConflicts()
    pub fn resolve_conflicts(&mut self) -> Result<bool, WfcError> {
        let conflict_cells = self.collect_conflict_cells();

        if conflict_cells.is_empty() {
            return Ok(true);
        }

        // 使用分层回溯解决所有冲突
        self.layered_backtrack_resolution(conflict_cells)
    }

    /// 收集所有冲突单元格
    fn collect_conflict_cells(&self) -> Vec<CellId> {
        self.wfc_data
            .iter()
            .filter(|(_, data)| data.state == CellState::Conflict)
            .map(|(&cell_id, _)| cell_id)
            .collect()
    }

    /// 分层回溯解决，对应C++的resolveConflictsCell()
    ///
    /// 这是WFC系统的核心冲突修复机制，通过分层回溯来解决冲突。
    /// 不同于传统WFC的过程性回溯，这里的回溯是专门为解决冲突层而设计的。
    fn layered_backtrack_resolution(
        &mut self,
        conflict_cells: Vec<CellId>,
    ) -> Result<bool, WfcError> {
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
        depth: usize,
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

    /// 恢复单元格可能性，用于分层修复
    fn recover_cell_possibilities(
        &mut self,
        cell_id: CellId,
        layers: &[Vec<CellId>],
    ) -> Result<(), WfcError> {
        // 将layers转换为Vec<Vec<CellId>>用于find_in_2d_vector
        let layers_vec: Vec<Vec<CellId>> = layers.to_vec();

        let (cx, _) = find_in_2d_vector(&layers_vec, &cell_id).unwrap_or((layers.len(), 0));
        let neighbors = self.grid.get_neighbors(cell_id);
        let mut neighbor_possibilities = Vec::new();

        for neighbor in neighbors {
            let (nx, _) = find_in_2d_vector(&layers_vec, &neighbor).unwrap_or((layers.len(), 0));
            if nx >= cx {
                if let Some(neighbor_data) = self.wfc_data.get(&neighbor) {
                    neighbor_possibilities.push(neighbor_data.possibilities.clone());
                } else {
                    neighbor_possibilities.push(vec![]);
                }
            } else {
                neighbor_possibilities.push(vec![]);
            }
        }

        // 根据邻居约束恢复可能性
        let mut new_possibilities = Vec::new();
        for tile_id in self.tile_set.get_all_tile_ids() {
            if self
                .tile_set
                .judge_possibility(&neighbor_possibilities, tile_id)
            {
                new_possibilities.push(tile_id);
            }
        }

        // 计算新的熵值
        let new_entropy = self.calculate_entropy(&new_possibilities);

        // 确定新状态
        let new_state = if new_possibilities.is_empty() {
            CellState::Conflict
        } else {
            CellState::Uncollapsed
        };

        // 最后更新单元格数据
        let cell_data = self
            .wfc_data
            .get_mut(&cell_id)
            .ok_or(WfcError::CellNotFound(cell_id))?;

        cell_data.possibilities = new_possibilities;
        cell_data.entropy = new_entropy;
        cell_data.state = new_state;

        Ok(())
    }

    /// 构建下一层
    fn build_next_layer(&self, current_layer: &[CellId]) -> Result<Vec<CellId>, WfcError> {
        let mut next_layer = Vec::new();

        for &cell in current_layer {
            let neighbors = self.grid.get_neighbors(cell);
            for neighbor in neighbors {
                if let Some(neighbor_data) = self.wfc_data.get(&neighbor) {
                    if neighbor_data.state == CellState::Collapsed
                        && !next_layer.contains(&neighbor)
                    {
                        next_layer.push(neighbor);
                    }
                }
            }
        }

        Ok(next_layer)
    }

    /// 回溯求解算法，对应C++的retrospectiveGetSolution()
    ///
    /// 这是分层修复过程中使用的局部回溯算法，用于在冲突修复时寻找可行的瓷砖组合。
    /// 注意：这不是传统WFC的全局回溯，而是针对特定冲突层的局部求解。
    fn backtrack_solution(&mut self, cells: &[CellId], index: usize) -> Result<bool, WfcError> {
        if index >= cells.len() {
            return Ok(true);
        }

        let cell_id = cells[index];
        let cell_data = self
            .wfc_data
            .get(&cell_id)
            .ok_or(WfcError::CellNotFound(cell_id))?;

        if cell_data.possibilities.is_empty() {
            return Ok(false);
        }

        // 最后一个单元直接选择第一种可能性
        if index == cells.len() - 1 {
            let first_possibility = cell_data.possibilities[0];
            self.set_tile_for_cell(cell_id, first_possibility)?;
            return Ok(true);
        }

        // 保存当前状态
        let snapshot = self.create_snapshot();

        // 尝试每种可能性
        for &possibility in &cell_data.possibilities.clone() {
            if self.tile_is_compatible(possibility, cell_id)? {
                // 设置瓷砖
                self.set_tile_for_cell(cell_id, possibility)?;

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

    /// 创建系统快照，对应C++的getSystem()
    fn create_snapshot(&self) -> SystemSnapshot {
        SystemSnapshot {
            data: self.wfc_data.clone(),
            completed_count: self.completed_count,
        }
    }

    /// 恢复系统快照，对应C++的setSystem()
    fn restore_snapshot(&mut self, snapshot: SystemSnapshot) -> Result<(), WfcError> {
        self.wfc_data = snapshot.data;
        self.completed_count = snapshot.completed_count;
        Ok(())
    }

    /// 检查是否有冲突
    fn has_conflicts(&self) -> bool {
        self.wfc_data
            .values()
            .any(|data| data.state == CellState::Conflict)
    }
}

// =============================================================================
// 测试模块
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile_set::TileSet;

    // 测试用的简单瓷砖集
    struct TestTileSet {
        tiles: TileSet<&'static str>,
    }

    impl TestTileSet {
        pub fn new() -> Self {
            let mut tiles = TileSet::new();
            tiles.add_tile(vec!["A", "B", "C", "D"], 10);
            tiles.add_tile(vec!["B", "A", "D", "C"], 15);
            Self { tiles }
        }
    }

    impl TileSetVirtual<&'static str> for TestTileSet {
        fn build_tile_set(&mut self) -> Result<(), GridError> {
            // 已在new中构建
            Ok(())
        }

        fn judge_possibility(
            &self,
            _neighbor_possibilities: &[Vec<TileId>],
            _candidate: TileId,
        ) -> bool {
            // 简单返回true用于测试
            true
        }

        fn get_tile(&self, tile_id: TileId) -> Option<&Tile<&'static str>> {
            self.tiles.get_tile(tile_id)
        }

        fn get_tile_count(&self) -> usize {
            self.tiles.get_tile_count()
        }

        fn get_all_tile_ids(&self) -> Vec<TileId> {
            self.tiles.get_all_tile_ids()
        }
    }


    #[test]
    fn test_wfc_manager_creation() {
        let grid = GridSystem::new();
        let tile_set = Box::new(TestTileSet::new()) as Box<dyn TileSetVirtual<&'static str>>;

        let manager = WfcManager::new(grid, tile_set).unwrap();
        assert_eq!(manager.completed_count, 0);
        assert!(manager.is_complete()); // 空网格自动完成
    }

    #[test]
    fn test_wfc_states() {
        assert_eq!(CellState::Uncollapsed, CellState::Uncollapsed);
        assert_ne!(CellState::Uncollapsed, CellState::Collapsed);

        let data = CellWfcData::new(12345, vec![0, 1]);
        assert_eq!(data.state, CellState::Uncollapsed);
        assert_eq!(data.rand_seed, 12345);
        assert_eq!(data.possibilities.len(), 2);
    }
}
