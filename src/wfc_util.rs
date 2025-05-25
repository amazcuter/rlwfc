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
pub type CellId = NodeIndex;

/// 边ID，对应原C++的EdgeID = GraphEdge*
pub type EdgeId = EdgeIndex;

/// 瓷砖ID，基于索引的实现，对应原C++的TileID<EdgeData>
pub type TileId = usize;

/// 单元格集合，对应原C++的Cells = std::vector<CellID>
pub type Cells = Vec<CellId>;

/// 瓷砖集合，对应原C++的Tiles = std::vector<TileID<EdgeData>>
pub type Tiles = Vec<TileId>;

/// 边集合，对应原C++的Edges = std::vector<EdgeID>
pub type Edges = Vec<EdgeId>;

/// WFC系统使用的图类型 - 使用有向图实现方向感知
pub type WFCGraph = Graph<Cell, GraphEdge, Directed>;

// =============================================================================
// 基础数据结构
// =============================================================================

/// 单元格数据，对应原C++的Cell类
/// 在petgraph中作为节点数据，不需要存储边信息（由图管理）
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Cell {
    /// 可选的单元格ID，用于调试和查找
    pub id: Option<u32>,
    /// 可选的单元格名称
    pub name: Option<String>,
}

impl Cell {
    /// 创建新的空单元格
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建带ID的单元格
    pub fn with_id(id: u32) -> Self {
        Self {
            id: Some(id),
            name: None,
        }
    }

    /// 创建带名称的单元格
    pub fn with_name(name: String) -> Self {
        Self {
            id: None,
            name: Some(name),
        }
    }
}

/// 图边数据，对应原C++的GraphEdge类
/// 在petgraph中作为边数据，连接关系由petgraph管理
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GraphEdge {
    /// 可选的边权重（使用整数避免浮点数比较问题）
    pub weight: Option<i32>,
    /// 可选的边类型标识
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
pub trait DirectionTrait: Clone + Copy + PartialEq + Eq + std::hash::Hash + std::fmt::Debug {
    /// 将方向转换为邻居数组的索引
    /// 
    /// 返回Some(index)表示该方向对应neighbors()返回数组中的index位置
    /// 返回None表示该方向需要通过反向查找获得（即查找指向当前节点的边）
    /// 
    /// # 重要说明
    /// 
    /// 由于petgraph的neighbors()返回的是插入逆序，索引映射需要考虑这一点。
    /// 例如，如果边按顺序创建为[东, 南]，neighbors()返回[南, 东]，
    /// 那么东方向应该映射到索引1，南方向映射到索引0。
    fn to_neighbor_index(&self) -> Option<usize>;
    
    /// 获取相反方向
    /// 
    /// 用于反向查找时确定对应关系。例如：
    /// - 北 <-> 南  
    /// - 东 <-> 西
    /// - 对于某些特殊方向可能没有相反方向，返回None
    fn opposite(&self) -> Option<Self>;
    
    /// 获取该方向系统的所有方向
    /// 
    /// 用于枚举和验证，按照边创建的标准顺序返回
    fn all_directions() -> Vec<Self>;
    
    /// 方向的显示名称（用于调试）
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

/// 瓷砖类，对应原C++的Tile<EdgeData>模板类
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