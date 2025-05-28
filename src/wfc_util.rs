//! # WFC工具模块
//! 
//! 本模块定义了Wave Function Collapse (WFC)系统的基础类型、错误处理和核心抽象，
//! 是对原C++ WFCutil.h的Rust重写版本。
//! 
//! ## 模块概述
//! 
//! 本模块包含以下核心组件：
//! 
//! - **基础类型**：对应原C++的using类型别名
//! - **数据结构**：单元格、边、瓷砖等核心数据结构
//! - **错误处理**：类型安全的错误系统
//! - **方向系统**：支持方向感知的trait设计
//! - **工具函数**：常用的辅助函数
//! 
//! ## 与原C++的对应关系
//! 
//! | C++类型/概念 | Rust类型/概念 | 说明 |
//! |-------------|---------------|------|
//! | `CellID = Cell*` | [`CellId = NodeIndex`] | 单元格标识符 |
//! | `EdgeID = GraphEdge*` | [`EdgeId = EdgeIndex`] | 边标识符 |
//! | `TileID<EdgeData>` | [`TileId = usize`] | 瓷砖标识符 |
//! | `std::vector<CellID>` | [`Cells = Vec<CellId>`] | 单元格集合 |
//! | `Cell` class | [`Cell`] struct | 单元格数据 |
//! | `GraphEdge` class | [`GraphEdge`] struct | 边数据 |
//! | `Tile<EdgeData>` class | [`Tile<EdgeData>`] struct | 瓷砖模板 |
//! 
//! ## 设计改进
//! 
//! ### 1. 类型安全
//! 
//! - 使用强类型别名替代原C++的指针类型
//! - 编译时类型检查，避免类型混淆
//! - 泛型设计支持不同的边数据类型
//! 
//! ### 2. 内存安全
//! 
//! - 自动内存管理，无需手动释放
//! - 借用检查器防止数据竞争
//! - 使用索引替代指针，避免悬垂指针
//! 
//! ### 3. 错误处理
//! 
//! - 使用`Result`类型进行错误处理
//! - 详细的错误分类，替代原C++的异常机制
//! - 所有可能的错误情况都有明确的类型表示
//! 
//! ### 4. 方向系统
//! 
//! 引入了原C++中没有的方向感知能力：
//! 
//! - [`DirectionTrait`]：通用的方向抽象
//! - [`Direction4`]：四方向网格的具体实现
//! - 支持编译时方向验证和运行时方向查询
//! 
//! ## 使用示例
//! 
//! ### 基础类型使用
//! 
//! ```rust
//! use rlwfc::{Cell, GraphEdge, Tile, Direction4, DirectionTrait};
//! 
//! // 创建单元格
//! let cell = Cell::with_id(1);
//! 
//! // 创建边
//! let edge = GraphEdge::with_weight(10);
//! 
//! // 创建瓷砖
//! let tile = Tile::new(0, 5, vec!["A", "B", "C", "D"]);
//! 
//! // 使用方向
//! let direction = Direction4::East;
//! println!("Direction: {}", direction.name());
//! ```
//! 
//! ### 方向系统使用
//! 
//! ```rust
//! use rlwfc::{Direction4, DirectionTrait};
//! 
//! // 获取邻居索引映射
//! let east_index = Direction4::East.to_neighbor_index();
//! 
//! // 获取相反方向
//! let west = Direction4::East.opposite().unwrap();
//! 
//! // 枚举所有方向
//! for direction in Direction4::all_directions() {
//!     println!("Direction: {}", direction.name());
//! }
//! ```
//! 
//! ## petgraph集成
//! 
//! 本模块设计为与petgraph图库无缝集成：
//! 
//! - 使用`NodeIndex`作为单元格ID
//! - 使用`EdgeIndex`作为边ID  
//! - 定义`WFCGraph`类型别名方便使用
//! - 支持有向图的方向感知功能

/**
 * @file wfc_util.rs
 * @author amazcuter (amazcuter@outlook.com)
 * @brief WFC系统基础类型定义 - Rust重写版本
 *        定义了基本概念和类型，对应原C++ WFCutil.h的功能
 * @version 1.0
 * @date 2025-01-25
 *
 * @copyright Copyright (c) 2025
 */

use petgraph::{Graph, Directed};
use petgraph::graph::{NodeIndex, EdgeIndex};

// =============================================================================
// 基础类型别名 - 对应原C++的using定义
// =============================================================================

/// 单元格ID，对应原C++的CellID = Cell*
/// 
/// 在Rust实现中，我们使用petgraph的`NodeIndex`来标识单元格，
/// 这比C++的指针类型更安全，避免了悬垂指针等内存安全问题。
pub type CellId = NodeIndex;

/// 边ID，对应原C++的EdgeID = GraphEdge*
/// 
/// 同样使用petgraph的`EdgeIndex`来标识边，提供类型安全的边引用。
pub type EdgeId = EdgeIndex;

/// 瓷砖ID，基于索引的实现，对应原C++的`TileID<EdgeData>`
/// 
/// 使用简单的usize索引来标识瓷砖，避免了C++模板的复杂性。
pub type TileId = usize;

/// 单元格集合，对应原C++的`Cells = std::vector<CellID>`
pub type Cells = Vec<CellId>;

/// 瓷砖集合，对应原C++的`Tiles = std::vector<TileID<EdgeData>>`
pub type Tiles = Vec<TileId>;

/// 边集合，对应原C++的`Edges = std::vector<EdgeID>`
pub type Edges = Vec<EdgeId>;

/// WFC系统使用的图类型 - 使用有向图实现方向感知
/// 
/// 这是整个WFC系统的核心数据结构类型别名。使用有向图的原因：
/// 
/// 1. **方向感知**：每条边都有明确的方向性
/// 2. **稳定顺序**：petgraph保证邻居返回的稳定顺序
/// 3. **高效查询**：O(1)的邻居查询操作
pub type WFCGraph = Graph<Cell, GraphEdge, Directed>;

// =============================================================================
// 基础数据结构
// =============================================================================

/// 单元格数据，对应原C++的Cell类
/// 
/// 在petgraph架构中，单元格作为图的节点数据存储。与原C++实现不同，
/// 这里不需要存储边信息，因为连接关系完全由petgraph的图结构管理。
/// 
/// ## 设计优势
/// 
/// - **简化结构**：移除了原C++中复杂的边管理逻辑
/// - **内存效率**：只存储必要的单元格属性
/// - **类型安全**：使用Option类型处理可选字段
/// 
/// ## 使用示例
/// 
/// ```rust
/// use rlwfc::Cell;
/// 
/// // 创建简单单元格
/// let cell1 = Cell::new();
/// 
/// // 创建带ID的单元格
/// let cell2 = Cell::with_id(42);
/// 
/// // 创建带名称的单元格
/// let cell3 = Cell::with_name("center_cell".to_string());
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Cell {
    /// 可选的单元格ID，用于调试和查找
    /// 
    /// 这是一个辅助字段，不影响图的结构，主要用于：
    /// - 调试输出
    /// - 与外部系统的ID映射
    /// - 测试验证
    pub id: Option<u32>,
    
    /// 可选的单元格名称
    /// 
    /// 提供人类可读的单元格标识，便于：
    /// - 调试和日志输出
    /// - 单元测试中的验证
    /// - 可视化工具的显示
    pub name: Option<String>,
}

impl Cell {
    /// 创建新的空单元格
    /// 
    /// 创建一个没有ID和名称的默认单元格，适用于简单的图构建场景。
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use rlwfc::Cell;
    /// 
    /// let cell = Cell::new();
    /// assert!(cell.id.is_none());
    /// assert!(cell.name.is_none());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建带ID的单元格
    /// 
    /// 创建一个带有数字ID的单元格，便于与外部系统的ID进行映射。
    /// 
    /// # 参数
    /// 
    /// * `id` - 单元格的数字标识符
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use rlwfc::Cell;
    /// 
    /// let cell = Cell::with_id(42);
    /// assert_eq!(cell.id, Some(42));
    /// ```
    pub fn with_id(id: u32) -> Self {
        Self {
            id: Some(id),
            name: None,
        }
    }

    /// 创建带名称的单元格
    /// 
    /// 创建一个带有字符串名称的单元格，便于调试和测试。
    /// 
    /// # 参数
    /// 
    /// * `name` - 单元格的字符串名称
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use rlwfc::Cell;
    /// 
    /// let cell = Cell::with_name("center".to_string());
    /// assert_eq!(cell.name, Some("center".to_string()));
    /// ```
    pub fn with_name(name: String) -> Self {
        Self {
            id: None,
            name: Some(name),
        }
    }
}

/// 图边数据，对应原C++的GraphEdge类
/// 
/// 在petgraph架构中，边数据存储与边关联的附加信息。与原C++实现不同，
/// 边的连接关系（from/to）完全由petgraph的图结构管理，这里只存储边的属性。
/// 
/// ## 设计考虑
/// 
/// - **权重类型**：使用整数而非浮点数，避免浮点数比较的精度问题
/// - **可选字段**：所有字段都是可选的，支持轻量级边创建
/// - **类型标识**：支持不同类型的边（如路径边、约束边等）
/// 
/// ## 使用场景
/// 
/// ```rust
/// use rlwfc::GraphEdge;
/// 
/// // 简单边
/// let simple_edge = GraphEdge::new();
/// 
/// // 带权重的边
/// let weighted_edge = GraphEdge::with_weight(10);
/// 
/// // 带类型的边
/// let typed_edge = GraphEdge::with_type("path".to_string());
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GraphEdge {
    /// 可选的边权重（使用整数避免浮点数比较问题）
    /// 
    /// 权重可用于：
    /// - 路径查找算法的成本计算
    /// - 连接强度的表示
    /// - 优先级排序
    pub weight: Option<i32>,
    
    /// 可选的边类型标识
    /// 
    /// 类型标识可用于：
    /// - 区分不同种类的连接
    /// - 过滤特定类型的边
    /// - 调试和可视化
    pub edge_type: Option<String>,
}

impl GraphEdge {
    /// 创建新的空边
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建带权重的边
    pub fn with_weight(weight: i32) -> Self {
        Self {
            weight: Some(weight),
            edge_type: None,
        }
    }

    /// 创建带类型的边
    pub fn with_type(edge_type: String) -> Self {
        Self {
            weight: None,
            edge_type: Some(edge_type),
        }
    }
}

// =============================================================================
// 错误处理
// =============================================================================

/// 网格系统错误类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridError {
    /// 尝试创建自循环边
    SelfLoop,
    /// 边已存在
    EdgeAlreadyExists,
    /// 节点不存在
    NodeNotFound,
    /// 边不存在  
    EdgeNotFound,
    /// 索引越界
    IndexOutOfBounds,
    /// 图容量不足
    CapacityExhausted,
    /// 方向无效
    InvalidDirection,
}

impl std::fmt::Display for GridError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GridError::SelfLoop => write!(f, "Cannot create self-loop edge"),
            GridError::EdgeAlreadyExists => write!(f, "Edge already exists"),
            GridError::NodeNotFound => write!(f, "Node not found"),
            GridError::EdgeNotFound => write!(f, "Edge not found"),
            GridError::IndexOutOfBounds => write!(f, "Index out of bounds"),
            GridError::CapacityExhausted => write!(f, "Graph capacity exhausted"),
            GridError::InvalidDirection => write!(f, "Invalid direction"),
        }
    }
}

impl std::error::Error for GridError {}

// =============================================================================
// 方向系统
// =============================================================================

/// 方向trait - 泛型设计，适配各种网格系统
/// 
/// 这是WFC系统中方向感知功能的核心抽象。与原C++实现不同，
/// Rust版本引入了完整的方向系统来支持方向感知的图操作。
/// 
/// ## 设计理念
/// 
/// ### 1. 方向到索引的映射
/// 
/// 核心创新是利用petgraph有向图的稳定邻居顺序来实现零成本的方向识别：
/// 
/// - petgraph的`neighbors()`返回邻居的顺序是插入顺序的逆序
/// - 通过标准化边创建顺序，可以建立方向到索引的固定映射
/// - 这样就能通过方向名称直接获取对应的邻居
/// 
/// ### 2. 双向映射策略
/// 
/// 对于完整的双向连接（如2D网格），采用以下策略：
/// 
/// - **前向方向**：直接从`neighbors()`结果获取（如东、南）
/// - **反向方向**：通过反向查找获取（如西、北）
/// 
/// ### 3. 类型约束
/// 
/// 要求实现类型满足以下约束：
/// - `Clone + Copy`：值语义，便于传递
/// - `PartialEq + Eq`：支持比较操作
/// - `Hash`：支持在HashMap等容器中使用
/// - `Debug`：便于调试输出
/// 
/// ## 使用模式
/// 
/// ```rust
/// use rlwfc::{Direction4, DirectionTrait};
/// 
/// // 获取方向对应的邻居索引
/// if let Some(index) = Direction4::East.to_neighbor_index() {
///     // 可以直接从neighbors()[index]获取该方向的邻居
///     println!("East neighbor at index {}", index);
/// }
/// 
/// // 获取相反方向
/// let west = Direction4::East.opposite().unwrap();
/// assert_eq!(west, Direction4::West);
/// 
/// // 枚举所有方向
/// for direction in Direction4::all_directions() {
///     println!("Direction: {}", direction.name());
/// }
/// ```
/// 
/// ## 扩展性
/// 
/// 这个trait设计支持多种网格类型：
/// 
/// - **2D四方向**：东南西北（已实现为`Direction4`）
/// - **2D八方向**：包含对角线方向
/// - **六角形网格**：六个方向
/// - **3D网格**：包含上下方向
/// - **自定义拓扑**：任意连接模式
pub trait DirectionTrait: Clone + Copy + PartialEq + Eq + std::hash::Hash + std::fmt::Debug {
    /// 将方向转换为邻居数组的索引
    /// 
    /// 这是方向感知系统的核心方法。它建立了方向名称到`neighbors()`返回数组索引的映射。
    /// 
    /// # 返回值
    /// 
    /// - `Some(index)` - 该方向对应`neighbors()`结果中的索引位置
    /// - `None` - 该方向需要通过反向查找获得（查找指向当前节点的边）
    /// 
    /// # 重要说明
    /// 
    /// 由于petgraph的`neighbors()`返回的是插入逆序，索引映射需要考虑这一点：
    /// 
    /// ```text
    /// 边创建顺序: [东, 南, 西, 北]
    /// neighbors(): [北, 西, 南, 东]  // 逆序
    /// 索引映射:    [0,  1,  2,  3]
    /// ```
    /// 
    /// 因此：
    /// - 东方向 -> 索引3
    /// - 南方向 -> 索引2  
    /// - 西方向 -> 索引1
    /// - 北方向 -> 索引0
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use rlwfc::{Direction4, DirectionTrait};
    /// 
    /// // 直接可获取的方向
    /// assert_eq!(Direction4::East.to_neighbor_index(), Some(1));
    /// assert_eq!(Direction4::South.to_neighbor_index(), Some(0));
    /// 
    /// // 需要反向查找的方向
    /// assert_eq!(Direction4::West.to_neighbor_index(), None);
    /// assert_eq!(Direction4::North.to_neighbor_index(), None);
    /// ```
    fn to_neighbor_index(&self) -> Option<usize>;
    
    /// 获取相反方向
    /// 
    /// 用于反向查找时确定对应关系，也用于双向连接的创建。
    /// 
    /// # 返回值
    /// 
    /// - `Some(opposite)` - 该方向的相反方向
    /// - `None` - 该方向没有明确的相反方向（如某些特殊拓扑）
    /// 
    /// # 标准对应关系
    /// 
    /// - 北 ↔ 南  
    /// - 东 ↔ 西
    /// - 上 ↔ 下（3D情况）
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use rlwfc::{Direction4, DirectionTrait};
    /// 
    /// assert_eq!(Direction4::North.opposite(), Some(Direction4::South));
    /// assert_eq!(Direction4::East.opposite(), Some(Direction4::West));
    /// ```
    fn opposite(&self) -> Option<Self>;
    
    /// 获取该方向系统的所有方向
    /// 
    /// 返回该方向系统支持的所有方向，按照标准顺序排列。
    /// 这个顺序通常也是边创建的推荐顺序。
    /// 
    /// # 返回值
    /// 
    /// 包含所有方向的向量，按标准顺序排列。
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use rlwfc::{Direction4, DirectionTrait};
    /// 
    /// let directions = Direction4::all_directions();
    /// assert_eq!(directions.len(), 4);
    /// 
    /// // 遍历所有方向
    /// for direction in directions {
    ///     println!("Direction: {}", direction.name());
    /// }
    /// ```
    fn all_directions() -> Vec<Self>;
    
    /// 方向的显示名称（用于调试）
    /// 
    /// 返回该方向的人类可读名称，主要用于调试输出和日志记录。
    /// 
    /// # 返回值
    /// 
    /// 该方向的字符串名称（静态字符串）。
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use rlwfc::{Direction4, DirectionTrait};
    /// 
    /// assert_eq!(Direction4::North.name(), "North");
    /// assert_eq!(Direction4::East.name(), "East");
    /// ```
    fn name(&self) -> &'static str;
}

/// 四方向网格的标准实现
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction4 {
    East,  // 东
    South, // 南  
    West,  // 西
    North, // 北
}

impl DirectionTrait for Direction4 {
    fn to_neighbor_index(&self) -> Option<usize> {
        match self {
            // 由于neighbors()返回逆序，且标准创建顺序为[东, 南]
            // 所以neighbors()返回[南, 东]，映射为：
            Direction4::South => Some(0), // 南在索引0
            Direction4::East => Some(1),  // 东在索引1
            // 西和北需要反向查找
            Direction4::West | Direction4::North => None,
        }
    }
    
    fn opposite(&self) -> Option<Self> {
        match self {
            Direction4::East => Some(Direction4::West),
            Direction4::West => Some(Direction4::East),
            Direction4::North => Some(Direction4::South),
            Direction4::South => Some(Direction4::North),
        }
    }
    
    fn all_directions() -> Vec<Self> {
        vec![Direction4::East, Direction4::South, Direction4::West, Direction4::North]
    }
    
    fn name(&self) -> &'static str {
        match self {
            Direction4::East => "East",
            Direction4::South => "South", 
            Direction4::West => "West",
            Direction4::North => "North",
        }
    }
}

// =============================================================================
// 瓷砖系统
// =============================================================================

/// 瓷砖类，对应原C++的`Tile<EdgeData>`模板类
#[derive(Debug, Clone, PartialEq)]
pub struct Tile<EdgeData> 
where 
    EdgeData: Clone + PartialEq + std::fmt::Debug
{
    /// 瓷砖ID
    pub id: TileId,
    /// 权重，对应原C++的weight字段
    pub weight: i32,
    /// 边信息，对应原C++的edge字段
    pub edges: Vec<EdgeData>,
}

impl<EdgeData> Tile<EdgeData> 
where 
    EdgeData: Clone + PartialEq + std::fmt::Debug
{
    /// 创建新瓷砖
    pub fn new(id: TileId, weight: i32, edges: Vec<EdgeData>) -> Self {
        Self { id, weight, edges }
    }
    
    /// 检查与另一个瓷砖的兼容性
    /// 对应原C++中可能的兼容性检查逻辑
    pub fn is_compatible_with(&self, other: &Self, direction: usize) -> bool {
        // 实现兼容性检查逻辑
        if direction < self.edges.len() && direction < other.edges.len() {
            // 简单的边匹配检查，可以根据具体需求扩展
            self.edges[direction] == other.edges[direction]
        } else {
            false
        }
    }
    
    /// 获取指定方向的边数据
    pub fn get_edge(&self, direction: usize) -> Option<&EdgeData> {
        self.edges.get(direction)
    }
    
    /// 获取可变边数据
    pub fn get_edge_mut(&mut self, direction: usize) -> Option<&mut EdgeData> {
        self.edges.get_mut(direction)
    }
    
    /// 获取边数量
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

// =============================================================================
// 工具函数
// =============================================================================

/// 在二维向量中查找元素，对应原C++的findIn2DVector函数
pub fn find_in_2d_vector<T>(vec_2d: &Vec<Vec<T>>, target: &T) -> Option<(usize, usize)>
where
    T: PartialEq,
{
    for (i, row) in vec_2d.iter().enumerate() {
        for (j, item) in row.iter().enumerate() {
            if item == target {
                return Some((i, j));
            }
        }
    }
    None
}

// =============================================================================
// 测试模块
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_creation() {
        let cell1 = Cell::new();
        let cell2 = Cell::with_id(42);
        let cell3 = Cell::with_name("test_cell".to_string());

        assert_eq!(cell1.id, None);
        assert_eq!(cell2.id, Some(42));
        assert_eq!(cell3.name, Some("test_cell".to_string()));
    }

    #[test]
    fn test_direction4() {
        assert_eq!(Direction4::East.opposite(), Some(Direction4::West));
        assert_eq!(Direction4::North.opposite(), Some(Direction4::South));
        assert_eq!(Direction4::East.to_neighbor_index(), Some(1));
        assert_eq!(Direction4::South.to_neighbor_index(), Some(0));
        assert_eq!(Direction4::West.to_neighbor_index(), None);
    }

    #[test]
    fn test_tile() {
        let tile = Tile::new(0, 10, vec!["A", "B", "C", "D"]);
        assert_eq!(tile.id, 0);
        assert_eq!(tile.weight, 10);
        assert_eq!(tile.edge_count(), 4);
        assert_eq!(tile.get_edge(0), Some(&"A"));
    }

    #[test]
    fn test_find_in_2d_vector() {
        let vec_2d = vec![
            vec![1, 2, 3],
            vec![4, 5, 6],
            vec![7, 8, 9],
        ];
        
        assert_eq!(find_in_2d_vector(&vec_2d, &5), Some((1, 1)));
        assert_eq!(find_in_2d_vector(&vec_2d, &1), Some((0, 0)));
        assert_eq!(find_in_2d_vector(&vec_2d, &10), None);
    }
} 