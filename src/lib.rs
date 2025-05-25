//! # RLWFC - Rust Wave Function Collapse Library
//! 
//! 这是一个基于Rust实现的Wave Function Collapse (WFC)算法库，
//! 使用petgraph作为底层图数据结构，提供类型安全和高性能的WFC实现。
//! 
//! ## 主要模块
//! 
//! - [`wfc_util`]: 基础类型定义、错误处理和方向系统
//! - [`grid_system`]: 网格系统实现，提供图操作和方向感知功能
//! 
//! ## 快速开始
//! 
//! ```rust
//! use RLWFC::{GridSystem, Cell, Direction4};
//! 
//! let mut grid = GridSystem::new();
//! let cell1 = grid.add_cell(Cell::new());
//! let cell2 = grid.add_cell(Cell::new());
//! 
//! // 创建边连接
//! grid.create_edge(cell1, cell2).unwrap();
//! 
//! // 获取邻居
//! let neighbors = grid.get_neighbors(cell1);
//! println!("Cell {:?} has {} neighbors", cell1, neighbors.len());
//! ```

/**
 * @file lib.rs
 * @author amazcuter (amazcuter@outlook.com)
 * @brief RLWFC - Rust Wave Function Collapse Library
 *        WFC系统的Rust实现，基于petgraph图库
 * @version 1.0
 * @date 2025-01-25
 *
 * @copyright Copyright (c) 2025
 */

pub mod wfc_util;
pub mod grid_system;

// 重新导出主要类型，方便使用
pub use wfc_util::{
    // 基础类型
    CellId, EdgeId, TileId, Cells, Tiles, Edges, WFCGraph,
    
    // 数据结构
    Cell, GraphEdge, Tile,
    
    // 错误处理
    GridError,
    
    // 方向系统
    DirectionTrait, Direction4,
    
    // 工具函数
    find_in_2d_vector,
};

pub use grid_system::GridSystem; 