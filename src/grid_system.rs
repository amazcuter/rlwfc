//! # 网格系统模块
//! 
//! 本模块提供了WFC（Wave Function Collapse）系统的网格系统实现，是对原C++代码的Rust重写版本。
//! 主要包含两个核心组件：
//! 
//! - [`GridBuilder`] trait：对应原C++的`buildGridSystem()`纯虚函数
//! - [`GridSystem`] 结构体：对应原C++的`GridSystem`类
//! 
//! ## 设计理念
//! 
//! ### GridBuilder Trait 设计
//! 
//! `GridBuilder` trait 是对原C++代码中 `buildGridSystem()` 纯虚函数的Rust实现，
//! 提供了一种类型安全、可组合的方式来构建不同类型的网格系统。
//! 
//! #### 原C++设计回顾
//! 
//! 在原C++代码中：
//! 
//! ```cpp
//! class GridSystem {
//!     // 建立网格系统纯虚函数
//!     virtual void buildGridSystem() = 0;
//!     
//!     // 其他方法...
//! };
//! 
//! class Orthogonal2DGrid : public GridSystem {
//!     virtual void buildGridSystem() override {
//!         // 具体的2D正交网格构建逻辑
//!     }
//! };
//! ```
//! 
//! #### Rust实现的改进
//! 
//! 相比C++的继承机制，Rust的trait设计提供了以下优势：
//! 
//! 1. **类型安全**：编译时错误检查，没有运行时类型转换
//! 2. **组合而非继承**：避免了复杂的继承层次，更好的代码复用
//! 3. **内存安全**：自动内存管理，借用检查器防止数据竞争
//! 4. **更灵活的设计**：可以在同一个GridSystem上使用不同的builder
//! 
//! ## 使用示例
//! 
//! ### 基本用法
//! 
//! ```rust
//! use rlwfc::{GridSystem, Cell, Direction4};
//! 
//! // 创建网格系统
//! let mut grid = GridSystem::new();
//! 
//! // 添加单元格
//! let cell1 = grid.add_cell(Cell::with_id(1));
//! let cell2 = grid.add_cell(Cell::with_id(2));
//! 
//! // 创建连接
//! grid.create_edge(cell1, cell2).unwrap();
//! 
//! // 获取邻居
//! let neighbors = grid.get_neighbors(cell1);
//! println!("Cell {:?} has {} neighbors", cell1, neighbors.len());
//! 
//! // 方向感知查询
//! if let Some(neighbor) = grid.get_neighbor_by_direction(cell1, Direction4::East) {
//!     println!("Eastern neighbor: {:?}", neighbor);
//! }
//! ```
//! 
//! ### 使用GridBuilder构建复杂网格
//! 
//! ```rust
//! use rlwfc::{GridSystem, GridBuilder, Cell, GridError};
//! 
//! // 自定义网格构建器
//! struct LinearGridBuilder {
//!     length: usize,
//! }
//! 
//! impl GridBuilder for LinearGridBuilder {
//!     fn build_grid_system(&mut self, grid: &mut GridSystem) -> Result<(), GridError> {
//!         // 创建线性连接的单元格
//!         let mut cells = Vec::new();
//!         for i in 0..self.length {
//!             let cell = grid.add_cell(Cell::with_id(i as u32));
//!             cells.push(cell);
//!         }
//!         
//!         // 创建线性连接
//!         for i in 0..self.length - 1 {
//!             grid.create_edge(cells[i], cells[i + 1])?;
//!         }
//!         
//!         Ok(())
//!     }
//!     
//!     fn get_grid_type_name(&self) -> &'static str {
//!         "LinearGrid"
//!     }
//! }
//! 
//! // 使用builder创建网格
//! let builder = LinearGridBuilder { length: 5 };
//! let grid = GridSystem::from_builder(builder).unwrap();
//! ```
//! 
//! ## 方向感知系统
//! 
//! 本模块的一个重要特性是方向感知能力，这是基于petgraph有向图的特性实现的：
//! 
//! - 使用有向图存储单元格连接
//! - 利用petgraph的稳定邻居顺序（按插入逆序）
//! - 通过[`DirectionTrait`]映射方向到邻居索引
//! 
//! 这使得可以通过方向名称直接查询邻居，而不需要额外的方向信息存储。

/**
 * @file grid_system.rs
 * @author amazcuter (amazcuter@outlook.com)
 * @brief WFC系统网格系统 - Rust重写版本
 *        对应原C++ GridSystem.h的功能，提供图操作和方向感知能力
 * @version 1.0
 * @date 2025-01-25
 *
 * @copyright Copyright (c) 2025
 */

use crate::wfc_util::*;
use petgraph::Graph;
use std::collections::HashMap;

// =============================================================================
// GridBuilder Trait - 对应C++的buildGridSystem虚函数
// =============================================================================

/// 网格构建器trait，对应原C++的buildGridSystem纯虚函数
/// 
/// 这个trait定义了构建不同类型网格的统一接口。每个具体的网格类型
/// （如2D正交网格、六角形网格、3D网格等）都应该实现这个trait。
/// 
/// # ⚠️ 关键约束：边创建顺序
/// 
/// 实现者**必须确保**为每个单元格按**相同的方向顺序**创建边，这对
/// 方向识别系统的正确性至关重要。
/// 
/// ## 边创建顺序的重要性
/// 
/// - **方向识别依赖**：`Direction4::to_neighbor_index()` 的映射依赖于固定的边创建顺序
/// - **petgraph特性**：利用 `neighbors()` 返回逆序的稳定性
/// - **全局一致性**：所有单元格必须使用相同的边创建顺序
/// 
/// ## 推荐的边创建顺序
/// 
/// 对于2D网格，推荐按以下顺序为每个单元格创建边：
/// 
/// 1. **东向边** (East) - 如果有东邻居
/// 2. **南向边** (South) - 如果有南邻居  
/// 3. **西向边** (West) - 如果有西邻居
/// 4. **北向边** (North) - 如果有北邻居
/// 
/// 这样 `neighbors()` 将返回 `[北, 西, 南, 东]` (逆序)，符合 `Direction4` 的索引映射。
/// 
/// ## 与原C++设计的对比
/// 
/// | 特性 | C++ | Rust |
/// |------|-----|------|
/// | 多态机制 | 虚函数继承 | Trait系统 |
/// | 类型安全 | 运行时检查 | 编译时检查 |
/// | 内存管理 | 手动管理 | 自动管理 |
/// | 错误处理 | 异常/返回码 | Result类型 |
/// | 扩展性 | 继承层次 | 组合设计 |
/// 
/// ## 实现指南
/// 
/// 实现这个trait时，`build_grid_system`方法应该：
/// 
/// 1. 创建所有需要的单元格
/// 2. **按固定顺序**建立单元格之间的连接关系
/// 3. 设置特定网格类型的属性
/// 4. 返回构建结果（成功或错误）
/// 
/// ## 示例实现
/// 
/// ```rust,no_run
/// use rlwfc::{GridSystem, GridBuilder, GridError, Cell};
/// 
/// struct Simple2DGrid {
///     width: usize,
///     height: usize,
/// }
/// 
/// impl GridBuilder for Simple2DGrid {
///     fn build_grid_system(&mut self, grid: &mut GridSystem) -> Result<(), GridError> {
///         // 1. 创建所有单元格
///         let mut cells = vec![vec![]; self.height];
///         for y in 0..self.height {
///             cells[y] = Vec::with_capacity(self.width);
///             for x in 0..self.width {
///                 let cell_id = grid.add_cell_with_name(
///                     Cell::with_id((y * self.width + x) as u32),
///                     format!("cell_{}_{}", x, y)
///                 );
///                 cells[y].push(cell_id);
///             }
///         }
/// 
///         // 2. 按固定顺序创建连接 - 这是关键！
///         for y in 0..self.height {
///             for x in 0..self.width {
///                 let current = cells[y][x];
///                 
///                 // 必须按相同顺序为每个单元格创建边
///                 
///                 // 1. 东向边
///                 if x + 1 < self.width {
///                     grid.create_edge(current, cells[y][x + 1])?;
///                 }
///                 
///                 // 2. 南向边
///                 if y + 1 < self.height {
///                     grid.create_edge(current, cells[y + 1][x])?;
///                 }
///                 
///                 // 3. 西向边
///                 if x > 0 {
///                     grid.create_edge(current, cells[y][x - 1])?;
///                 }
///                 
///                 // 4. 北向边
///                 if y > 0 {
///                     grid.create_edge(current, cells[y - 1][x])?;
///                 }
///             }
///         }
/// 
///         Ok(())
///     }
///     
///     fn get_dimensions(&self) -> Vec<usize> {
///         vec![self.width, self.height]
///     }
///     
///     fn get_grid_type_name(&self) -> &'static str {
///         "Simple2DGrid"
///     }
/// }
/// ```
pub trait GridBuilder {
    /// 构建网格系统，对应原C++的buildGridSystem()纯虚函数
    /// 
    /// 这个方法是GridBuilder trait的核心，负责实际的网格构建逻辑。
    /// 不同的网格类型会有不同的实现方式。
    /// 
    /// # 参数
    /// 
    /// * `grid` - 要构建的网格系统，方法应该在其中添加单元格和边
    /// 
    /// # 返回值
    /// 
    /// * `Ok(())` - 构建成功
    /// * `Err(GridError)` - 构建过程中遇到错误
    /// 
    /// # 错误情况
    /// 
    /// - 创建重复边时返回`GridError::EdgeAlreadyExists`
    /// - 尝试创建自循环时返回`GridError::SelfLoop`
    /// - 其他图操作错误
    fn build_grid_system(&mut self, grid: &mut GridSystem) -> Result<(), GridError>;
    
    /// 获取网格的维度信息（可选实现）
    /// 
    /// 返回网格的维度信息，例如：
    /// - 2D网格：`[width, height]`
    /// - 3D网格：`[width, height, depth]`
    /// - 线性网格：`[length]`
    /// 
    /// 默认实现返回空向量，表示未指定维度。
    fn get_dimensions(&self) -> Vec<usize> {
        vec![]
    }
    
    /// 获取网格类型的名称（可选实现）
    /// 
    /// 返回一个描述性的网格类型名称，用于调试和日志输出。
    /// 
    /// 默认实现返回"CustomGrid"。
    fn get_grid_type_name(&self) -> &'static str {
        "CustomGrid"
    }
}

// =============================================================================
// GridSystem 核心结构
// =============================================================================

/// 网格系统类，对应原C++的GridSystem类
/// 
/// `GridSystem`是WFC系统的核心数据结构，使用有向图来表示单元格之间的连接关系。
/// 与原C++实现相比，这个Rust版本提供了以下改进：
/// 
/// ## 核心特性
/// 
/// ### 1. 方向感知能力
/// 
/// 使用有向图实现零成本的方向识别：
/// - 每条边代表一个方向性连接
/// - 利用petgraph的稳定邻居顺序
/// - 通过[`DirectionTrait`]实现方向到索引的映射
/// 
/// ### 2. 类型安全
/// 
/// - 所有操作都有明确的类型约束
/// - 编译时错误检查
/// - 使用`Result`类型进行错误处理
/// 
/// ### 3. 内存安全
/// 
/// - 自动内存管理，无需手动释放
/// - 借用检查器防止数据竞争
/// - 无空指针或野指针风险
/// 
/// ## 与原C++的API对应关系
/// 
/// | C++方法 | Rust方法 | 说明 |
/// |---------|----------|------|
/// | `CreateEdge(cellA, cellB)` | [`create_edge(from, to)`] | 创建有向边 |
/// | `getNeighbor(cell)` | [`get_neighbors(cell_id)`] | 获取邻居列表 |
/// | `findEdge(cellA, cellB)` | [`find_edge(from, to)`] | 查找边 |
/// | `getAllCells()` | [`get_all_cells()`] | 获取所有单元格 |
/// | `getCellsNum()` | [`get_cells_count()`] | 获取单元格数量 |
/// | `buildGridSystem()` | [`build_with(builder)`] | 构建网格系统 |
/// 
/// ## 使用模式
/// 
/// ### 方式一：分步构建
/// 
/// ```rust,no_run
/// use rlwfc::{GridSystem, GridBuilder, GridError};
/// 
/// struct CustomGridBuilder;
/// 
/// impl GridBuilder for CustomGridBuilder {
///     fn build_grid_system(&mut self, grid: &mut GridSystem) -> Result<(), GridError> {
///         Ok(())
///     }
/// }
/// 
/// let mut grid = GridSystem::new();
/// let builder = CustomGridBuilder;
/// grid.build_with(builder)?;
/// # Ok::<(), GridError>(())
/// ```
/// 
/// ### 方式二：直接构建
/// 
/// ```rust,no_run
/// use rlwfc::{GridSystem, GridBuilder, GridError};
/// 
/// struct CustomGridBuilder;
/// 
/// impl GridBuilder for CustomGridBuilder {
///     fn build_grid_system(&mut self, grid: &mut GridSystem) -> Result<(), GridError> {
///         Ok(())
///     }
/// }
/// 
/// let builder = CustomGridBuilder;
/// let grid = GridSystem::from_builder(builder)?;
/// # Ok::<(), GridError>(())
/// ```
/// 
/// [`create_edge(from, to)`]: GridSystem::create_edge
/// [`get_neighbors(cell_id)`]: GridSystem::get_neighbors
/// [`find_edge(from, to)`]: GridSystem::find_edge
/// [`get_all_cells()`]: GridSystem::get_all_cells
/// [`get_cells_count()`]: GridSystem::get_cells_count
/// [`build_with(builder)`]: GridSystem::build_with
pub struct GridSystem {
    /// 底层图存储，使用有向图支持方向识别
    /// 
    /// 这是整个网格系统的核心数据结构。使用petgraph的有向图来存储：
    /// - 节点：代表网格中的单元格
    /// - 边：代表单元格之间的有向连接
    graph: WFCGraph,
    
    /// 可选的单元格名称映射，用于快速查找
    /// 
    /// 允许通过字符串名称快速查找单元格ID，便于调试和测试。
    /// 这是一个可选功能，不影响核心图操作的性能。
    cell_lookup: HashMap<String, CellId>,
}

impl GridSystem {
    /// 创建新的网格系统
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            cell_lookup: HashMap::new(),
        }
    }

    /// 创建带容量的网格系统
    pub fn with_capacity(nodes: usize, edges: usize) -> Self {
        Self {
            graph: Graph::with_capacity(nodes, edges),
            cell_lookup: HashMap::new(),
        }
    }

    /// 使用builder构建网格系统，对应原C++的多态buildGridSystem调用
    pub fn build_with<T: GridBuilder>(&mut self, mut builder: T) -> Result<(), GridError> {
        builder.build_grid_system(self)
    }

    /// 创建新的网格系统并立即使用builder构建
    pub fn from_builder<T: GridBuilder>(mut builder: T) -> Result<Self, GridError> {
        let mut grid = Self::new();
        builder.build_grid_system(&mut grid)?;
        Ok(grid)
    }

    // ==========================================================================
    // 基础图操作 - 对应原C++的核心方法
    // ==========================================================================

    /// 添加单元格，对应原C++中向cells_添加元素
    pub fn add_cell(&mut self, cell_data: Cell) -> CellId {
        self.graph.add_node(cell_data)
    }

    /// 添加带名称的单元格，支持按名称查找
    pub fn add_cell_with_name(&mut self, cell_data: Cell, name: String) -> CellId {
        let cell_id = self.add_cell(cell_data);
        self.cell_lookup.insert(name, cell_id);
        cell_id
    }

    /// 根据名称获取单元格ID
    pub fn get_cell_by_name(&self, name: &str) -> Option<CellId> {
        self.cell_lookup.get(name).copied()
    }

    /// 创建单向边，对应原C++的CreateEdge方法
    /// 
    /// # ⚠️ 重要：边创建顺序约束
    /// 
    /// 这个方法创建的是单向边，用于构建WFC系统需要的无向连接。为了确保
    /// 方向识别系统正常工作，**必须按全局一致的顺序**为每个单元格创建边。
    /// 
    /// ## 设计原理
    /// 
    /// - **无向连接**：WFC系统需要双向可达的连接
    /// - **方向识别**：通过边创建顺序和petgraph的逆序返回特性实现
    /// - **边对需求**：每个无向连接需要两条相对的有向边（A→B 和 B→A）
    /// 
    /// ## 正确使用模式
    /// 
    /// ```rust,no_run
    /// # use rlwfc::{GridSystem, GridBuilder, GridError};
    /// # struct MyBuilder;
    /// # impl GridBuilder for MyBuilder {
    /// #   fn build_grid_system(&mut self, grid: &mut GridSystem) -> Result<(), GridError> {
    /// // ✅ 正确：在GridBuilder中按全局一致顺序创建边
    /// for cell in all_cells {
    ///     // 按固定顺序为每个单元格创建边：东、南、西、北
    ///     if let Some(east) = get_east_neighbor(cell) {
    ///         grid.create_edge(cell, east)?;  // 1. 东向边
    ///     }
    ///     if let Some(south) = get_south_neighbor(cell) {
    ///         grid.create_edge(cell, south)?; // 2. 南向边
    ///     }
    ///     if let Some(west) = get_west_neighbor(cell) {
    ///         grid.create_edge(cell, west)?;  // 3. 西向边
    ///     }
    ///     if let Some(north) = get_north_neighbor(cell) {
    ///         grid.create_edge(cell, north)?; // 4. 北向边
    ///     }
    /// }
    /// #     Ok(())
    /// #   }
    /// # }
    /// ```
    /// 
    /// ## ❌ 错误使用模式
    /// 
    /// ```rust,no_run
    /// # use rlwfc::{GridSystem, CellId, GridError};
    /// # let mut grid = GridSystem::new();
    /// # let cell_a = grid.add_cell(Default::default());
    /// # let cell_b = grid.add_cell(Default::default());
    /// // ❌ 错误：随意创建边会破坏方向识别
    /// grid.create_edge(cell_a, cell_b)?;
    /// grid.create_edge(cell_b, cell_a)?; // 顺序可能不正确
    /// # Ok::<(), GridError>(())
    /// ```
    /// 
    /// ## 为什么不提供自动双向连接方法
    /// 
    /// 本库故意不提供 `create_undirected_connection()` 等便捷方法，因为：
    /// 
    /// 1. **顺序依赖**：方向识别完全依赖边创建的全局一致顺序
    /// 2. **应用层责任**：正确的边创建顺序只能由具体的网格构建逻辑确定
    /// 3. **错误预防**：避免提供可能破坏顺序一致性的便捷方法
    /// 
    /// # 参数
    /// 
    /// * `from` - 源单元格ID
    /// * `to` - 目标单元格ID
    /// 
    /// # 返回值
    /// 
    /// * `Ok(EdgeId)` - 成功创建的边ID
    /// * `Err(GridError)` - 创建失败的错误信息
    /// 
    /// # 错误情况
    /// 
    /// - `GridError::SelfLoop` - 尝试创建自循环边
    /// - `GridError::EdgeAlreadyExists` - 边已存在
    /// - `GridError::NodeNotFound` - 源或目标节点不存在
    pub fn create_edge(&mut self, from: CellId, to: CellId) -> Result<EdgeId, GridError> {
        // 检查自循环
        if from == to {
            return Err(GridError::SelfLoop);
        }

        // 检查节点是否存在
        if !self.graph.node_indices().any(|n| n == from) {
            return Err(GridError::NodeNotFound);
        }
        if !self.graph.node_indices().any(|n| n == to) {
            return Err(GridError::NodeNotFound);
        }

        // 检查边是否已存在（单向检查）
        if self.graph.find_edge(from, to).is_some() {
            return Err(GridError::EdgeAlreadyExists);
        }

        // 创建单向边：from指向to
        // 这与C++的CreateEdge(cellA, cellB)语义一致
        let edge_id = self.graph.add_edge(from, to, GraphEdge::new());
        Ok(edge_id)
    }

    /// 获取邻居，对应原C++的getNeighbor方法
    /// 
    /// 利用petgraph有向图的特性实现方向感知
    /// 返回从该节点出发的所有目标节点，按插入逆序排列
    pub fn get_neighbors(&self, cell_id: CellId) -> Vec<CellId> {
        // 在有向图中，neighbors()返回从该节点出发的所有边的目标节点
        // 顺序为边添加的逆序，这是petgraph的稳定行为
        self.graph.neighbors(cell_id).collect()
    }

    /// 查找边，对应原C++的findEdge方法
    pub fn find_edge(&self, from: CellId, to: CellId) -> Option<EdgeId> {
        self.graph.find_edge(from, to)
    }

    /// 获取所有单元格，对应原C++的getAllCells方法
    pub fn get_all_cells(&self) -> impl Iterator<Item = CellId> + '_ {
        self.graph.node_indices()
    }

    /// 获取单元格数量，对应原C++的getCellsNum方法
    pub fn get_cells_count(&self) -> usize {
        self.graph.node_count()
    }

    /// 获取边数量
    pub fn get_edges_count(&self) -> usize {
        self.graph.edge_count()
    }

    // ==========================================================================
    // 方向感知API - 新增的方向识别功能
    // ==========================================================================

    /// 基于方向获取特定邻居 - 核心的方向感知API
    pub fn get_neighbor_by_direction<D>(&self, cell_id: CellId, direction: D) -> Option<CellId> 
    where 
        D: DirectionTrait
    {
        let neighbors = self.get_neighbors(cell_id);
        
        // 根据方向trait的索引映射获取邻居
        if let Some(index) = direction.to_neighbor_index() {
            neighbors.get(index).copied()
        } else {
            // 如果索引映射返回None，需要反向查找
            self.find_incoming_neighbor_by_direction(cell_id, direction)
        }
    }

    /// 查找反向邻居（指向当前节点的邻居）
    fn find_incoming_neighbor_by_direction<D>(&self, cell_id: CellId, direction: D) -> Option<CellId> 
    where 
        D: DirectionTrait
    {
        // 对于需要反向查找的方向，遍历所有节点
        for node_id in self.graph.node_indices() {
            let neighbors = self.get_neighbors(node_id);
            
            // 检查该节点是否通过特定方向指向当前节点
            if let Some(opposite_direction) = direction.opposite() {
                if let Some(index) = opposite_direction.to_neighbor_index() {
                    if let Some(&neighbor) = neighbors.get(index) {
                        if neighbor == cell_id {
                            return Some(node_id);
                        }
                    }
                }
            }
        }
        None
    }

    /// 获取指定方向的所有邻居
    pub fn get_neighbors_by_direction<D>(&self, cell_id: CellId, direction: D) -> Vec<CellId>
    where 
        D: DirectionTrait
    {
        if let Some(neighbor) = self.get_neighbor_by_direction(cell_id, direction) {
            vec![neighbor]
        } else {
            vec![]
        }
    }

    // ==========================================================================
    // 图状态查询和验证
    // ==========================================================================

    /// 检查节点是否存在
    pub fn contains_cell(&self, cell_id: CellId) -> bool {
        self.graph.node_indices().any(|n| n == cell_id)
    }

    /// 检查边是否存在
    pub fn contains_edge(&self, from: CellId, to: CellId) -> bool {
        self.graph.find_edge(from, to).is_some()
    }

    /// 获取图的容量信息
    pub fn capacity(&self) -> (usize, usize) {
        self.graph.capacity()
    }

    /// 清空图
    pub fn clear(&mut self) {
        self.graph.clear();
        self.cell_lookup.clear();
    }

    /// 获取单元格的度数（连接数）
    pub fn get_cell_degree(&self, cell_id: CellId) -> usize {
        self.get_neighbors(cell_id).len()
    }

    // ==========================================================================
    // 验证和调试工具
    // ==========================================================================

    /// 验证网格结构的完整性
    pub fn validate_structure(&self) -> Result<(), GridError> {
        // 验证所有边的端点都存在
        for edge_id in self.graph.edge_indices() {
            if let Some((source, target)) = self.graph.edge_endpoints(edge_id) {
                if !self.contains_cell(source) {
                    return Err(GridError::NodeNotFound);
                }
                if !self.contains_cell(target) {
                    return Err(GridError::NodeNotFound);
                }
            }
        }
        Ok(())
    }

    /// 获取网格统计信息
    pub fn get_statistics(&self) -> String {
        format!(
            "GridSystem Statistics:\n  Nodes: {}\n  Edges: {}\n  Capacity: {:?}\n  Named cells: {}",
            self.get_cells_count(),
            self.get_edges_count(),
            self.capacity(),
            self.cell_lookup.len()
        )
    }

    /// 调试打印指定单元格的邻居信息
    pub fn debug_print_neighbors(&self, cell_id: CellId) {
        println!("Cell {:?} neighbors:", cell_id);
        let neighbors = self.get_neighbors(cell_id);
        for (i, neighbor) in neighbors.iter().enumerate() {
            println!("  [{}]: {:?}", i, neighbor);
        }
        
        // 测试Direction4的方向查询
        println!("  Direction queries:");
        for direction in Direction4::all_directions() {
            if let Some(neighbor) = self.get_neighbor_by_direction(cell_id, direction) {
                println!("    {}: {:?}", direction.name(), neighbor);
            } else {
                println!("    {}: None", direction.name());
            }
        }
    }

    /// 调试打印整个网格的信息
    pub fn debug_print_grid(&self) {
        println!("=== Grid System Debug Info ===");
        println!("{}", self.get_statistics());
        println!("\nAll cells:");
        for cell_id in self.get_all_cells() {
            let neighbors = self.get_neighbors(cell_id);
            println!("  {:?}: neighbors = {:?}", cell_id, neighbors);
        }
        
        println!("\nNamed cells:");
        for (name, cell_id) in &self.cell_lookup {
            println!("  '{}': {:?}", name, cell_id);
        }
    }
}

// =============================================================================
// 测试模块
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // 测试用的简单网格构建器
    struct SimpleGridBuilder {
        width: usize,
        height: usize,
    }

    impl SimpleGridBuilder {
        fn new(width: usize, height: usize) -> Self {
            Self { width, height }
        }
    }

    impl GridBuilder for SimpleGridBuilder {
        fn build_grid_system(&mut self, grid: &mut GridSystem) -> Result<(), GridError> {
            // 创建一个简单的width x height网格
            let mut cells = vec![vec![]; self.height];
            
            // Step 1: 创建所有单元格
            for y in 0..self.height {
                cells[y] = Vec::with_capacity(self.width);
                for x in 0..self.width {
                    let cell_id = grid.add_cell_with_name(
                        Cell::with_id((y * self.width + x) as u32),
                        format!("cell_{}_{}", x, y)
                    );
                    cells[y].push(cell_id);
                }
            }

            // Step 2: 创建连接（每个cell连接到右边和下面的邻居）
            for y in 0..self.height {
                for x in 0..self.width {
                    let current = cells[y][x];
                    
                    // 连接到右边
                    if x + 1 < self.width {
                        grid.create_edge(current, cells[y][x + 1])?;
                    }
                    
                    // 连接到下面
                    if y + 1 < self.height {
                        grid.create_edge(current, cells[y + 1][x])?;
                    }
                }
            }

            Ok(())
        }

        fn get_dimensions(&self) -> Vec<usize> {
            vec![self.width, self.height]
        }

        fn get_grid_type_name(&self) -> &'static str {
            "SimpleGrid"
        }
    }

    #[test]
    fn test_grid_system_creation() {
        let grid = GridSystem::new();
        assert_eq!(grid.get_cells_count(), 0);
        assert_eq!(grid.get_edges_count(), 0);
    }

    #[test]
    fn test_add_cells_and_edges() {
        let mut grid = GridSystem::new();
        
        // 添加单元格
        let cell1 = grid.add_cell(Cell::with_id(1));
        let cell2 = grid.add_cell(Cell::with_id(2));
        
        assert_eq!(grid.get_cells_count(), 2);
        
        // 创建边
        let _edge = grid.create_edge(cell1, cell2).unwrap();
        assert_eq!(grid.get_edges_count(), 1);
        
        // 检查邻居
        let neighbors = grid.get_neighbors(cell1);
        assert_eq!(neighbors.len(), 1);
        assert_eq!(neighbors[0], cell2);
    }

    #[test]
    fn test_direction_queries() {
        let mut grid = GridSystem::new();
        
        // 创建2x2网格
        let cells = vec![
            vec![grid.add_cell(Cell::with_id(0)), grid.add_cell(Cell::with_id(1))],
            vec![grid.add_cell(Cell::with_id(2)), grid.add_cell(Cell::with_id(3))],
        ];
        
        // 按标准顺序创建边：东向，然后南向
        let center = cells[0][0];
        let east = cells[0][1];
        let south = cells[1][0];
        
        grid.create_edge(center, east).unwrap();  // 东向边
        grid.create_edge(center, south).unwrap(); // 南向边
        
        // 测试方向查询
        assert_eq!(grid.get_neighbor_by_direction(center, Direction4::East), Some(east));
        assert_eq!(grid.get_neighbor_by_direction(center, Direction4::South), Some(south));
        assert_eq!(grid.get_neighbor_by_direction(center, Direction4::West), None);
        assert_eq!(grid.get_neighbor_by_direction(center, Direction4::North), None);
    }

    #[test]
    fn test_error_handling() {
        let mut grid = GridSystem::new();
        
        let cell1 = grid.add_cell(Cell::new());
        
        // 测试自循环错误
        assert_eq!(grid.create_edge(cell1, cell1), Err(GridError::SelfLoop));
        
        // 测试重复边错误
        let cell2 = grid.add_cell(Cell::new());
        grid.create_edge(cell1, cell2).unwrap();
        assert_eq!(grid.create_edge(cell1, cell2), Err(GridError::EdgeAlreadyExists));
    }

    #[test]
    fn test_named_cells() {
        let mut grid = GridSystem::new();
        
        let cell_id = grid.add_cell_with_name(Cell::new(), "test_cell".to_string());
        assert_eq!(grid.get_cell_by_name("test_cell"), Some(cell_id));
        assert_eq!(grid.get_cell_by_name("nonexistent"), None);
    }

    #[test]
    fn test_structure_validation() {
        let mut grid = GridSystem::new();
        
        let cell1 = grid.add_cell(Cell::new());
        let cell2 = grid.add_cell(Cell::new());
        grid.create_edge(cell1, cell2).unwrap();
        
        // 验证应该成功
        assert!(grid.validate_structure().is_ok());
    }

    #[test]
    fn test_grid_builder_trait() {
        // 测试使用builder构建网格
        let builder = SimpleGridBuilder::new(3, 2);
        let mut grid = GridSystem::new();
        
        // 使用builder构建网格
        grid.build_with(builder).unwrap();
        
        // 验证构建结果
        assert_eq!(grid.get_cells_count(), 6); // 3x2 = 6个单元格
        
        // 验证命名单元格
        assert!(grid.get_cell_by_name("cell_0_0").is_some());
        assert!(grid.get_cell_by_name("cell_2_1").is_some());
        assert!(grid.get_cell_by_name("cell_3_0").is_none()); // 超出范围
    }

    #[test]
    fn test_from_builder() {
        // 测试从builder直接创建网格
        let builder = SimpleGridBuilder::new(2, 2);
        let grid = GridSystem::from_builder(builder).unwrap();
        
        assert_eq!(grid.get_cells_count(), 4); // 2x2 = 4个单元格
        
        // 测试连接性：每个内部单元格应该有邻居
        let cell_0_0 = grid.get_cell_by_name("cell_0_0").unwrap();
        let neighbors = grid.get_neighbors(cell_0_0);
        assert_eq!(neighbors.len(), 2); // 连接到右边和下面
    }
} 