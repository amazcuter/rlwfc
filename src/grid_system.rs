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
// GridSystem 核心结构
// =============================================================================

/// 网格系统类，对应原C++的GridSystem类
/// 使用有向图实现方向感知的图操作
pub struct GridSystem {
    /// 底层图存储，使用有向图支持方向识别
    graph: WFCGraph,
    
    /// 可选的单元格名称映射，用于快速查找
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
    /// 这是方向感知的关键：只创建单向边，方向由from->to确定
    /// 与C++的CreateEdge(cellA, cellB)语义完全一致
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
// Default trait 实现
// =============================================================================

impl Default for GridSystem {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// 测试模块
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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
} 