//! # RLWFC - Rust Wave Function Collapse Library
//! 
//! 这是一个基于Rust实现的Wave Function Collapse (WFC)算法库，
//! 使用petgraph作为底层图数据结构，提供类型安全和高性能的WFC实现。
//! 
//! ## 库概述
//! 
//! RLWFC是对原C++ WFC系统的完整Rust重写，在保持API兼容性的同时，
//! 引入了现代Rust的设计理念和安全保证。
//! 
//! ### 核心特性
//! 
//! - **类型安全**：编译时保证类型正确性，避免运行时错误
//! - **内存安全**：自动内存管理，无悬垂指针和内存泄漏
//! - **方向感知**：创新的方向识别系统，支持空间方向查询
//! - **零成本抽象**：高级抽象不影响运行时性能
//! - **并发安全**：支持并发访问的数据结构设计
//! 
//! ### 设计原则
//! 
//! 1. **与原C++的兼容性**：保持核心API的一致性
//! 2. **Rust习惯用法**：遵循Rust社区的最佳实践
//! 3. **模块化设计**：清晰的职责分离和组合能力
//! 4. **可扩展性**：支持自定义网格类型和约束规则
//! 
//! ## 主要模块
//! 
//! ### [`wfc_util`] - 基础工具模块
//! 
//! 提供WFC系统的基础类型定义、错误处理和方向系统：
//! 
//! - **类型别名**：`CellId`, `EdgeId`, `TileId` 等核心类型
//! - **数据结构**：`Cell`, `GraphEdge`, `Tile` 等基础结构
//! - **错误处理**：`GridError` 枚举，提供详细的错误分类
//! - **方向系统**：`DirectionTrait` 和 `Direction4` 实现
//! 
//! ### [`grid_system`] - 网格系统模块
//! 
//! 实现了WFC系统的核心网格管理功能：
//! 
//! - **图操作**：基于petgraph的高效图操作
//! - **方向感知**：零成本的方向识别和查询
//! - **构建器模式**：`GridBuilder` trait支持多种网格类型
//! - **调试工具**：完整的验证和调试功能
//! 
//! ### [`tile_set`] - 瓷砖集模块
//! 
//! 实现了WFC算法的瓷砖管理和约束判断：
//! 
//! - **瓷砖管理**：高效的瓷砖存储和查询
//! - **约束判断**：灵活的约束规则实现
//! - **泛型设计**：支持任意类型的边数据
//! - **虚函数模拟**：trait系统替代C++虚函数
//! 
//! ## 快速开始
//! 
//! ### 基本图操作
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
//! // 创建边连接
//! grid.create_edge(cell1, cell2).unwrap();
//! 
//! // 获取邻居
//! let neighbors = grid.get_neighbors(cell1);
//! println!("Cell {:?} has {} neighbors", cell1, neighbors.len());
//! 
//! // 方向感知查询
//! if let Some(eastern_neighbor) = grid.get_neighbor_by_direction(cell1, Direction4::East) {
//!     println!("Eastern neighbor: {:?}", eastern_neighbor);
//! }
//! ```
//! 
//! ### 使用构建器创建复杂网格
//! 
//! ```rust
//! use rlwfc::{GridSystem, GridBuilder, Cell, GridError};
//! 
//! // 定义自定义网格构建器
//! struct RingGridBuilder {
//!     size: usize,
//! }
//! 
//! impl GridBuilder for RingGridBuilder {
//!     fn build_grid_system(&mut self, grid: &mut GridSystem) -> Result<(), GridError> {
//!         // 创建环形连接的单元格
//!         let mut cells = Vec::new();
//!         for i in 0..self.size {
//!             let cell = grid.add_cell(Cell::with_id(i as u32));
//!             cells.push(cell);
//!         }
//!         
//!         // 创建环形连接
//!         for i in 0..self.size {
//!             let next = (i + 1) % self.size;
//!             grid.create_edge(cells[i], cells[next])?;
//!         }
//!         
//!         Ok(())
//!     }
//! }
//! 
//! // 使用构建器
//! let builder = RingGridBuilder { size: 6 };
//! let grid = GridSystem::from_builder(builder).unwrap();
//! ```
//! 
//! ### 瓷砖系统使用
//! 
//! ```rust
//! use rlwfc::{TileSet, TileSetVirtual, TileId};
//! 
//! // 创建瓷砖集
//! let mut tiles = TileSet::new();
//! let tile_id = tiles.add_tile(vec!["grass", "water", "grass", "water"], 10);
//! 
//! // 获取瓷砖信息
//! if let Some(tile) = tiles.get_tile(tile_id) {
//!     println!("Tile has {} edges", tile.edge_count());
//! }
//! ```
//! 
//! ## 架构优势
//! 
//! ### 与原C++版本对比
//! 
//! | 特性 | C++ | Rust |
//! |------|-----|------|
//! | 内存安全 | 手动管理 | 自动保证 |
//! | 类型安全 | 运行时检查 | 编译时保证 |
//! | 错误处理 | 异常/返回码 | Result类型 |
//! | 多态 | 虚函数继承 | Trait组合 |
//! | 并发 | 手动同步 | 编译时保证 |
//! | 方向感知 | 无 | 零成本实现 |
//! 
//! ### 性能特点
//! 
//! - **零成本抽象**：高级功能不增加运行时开销
//! - **内存效率**：紧凑的数据布局，minimal内存占用
//! - **缓存友好**：连续内存访问模式
//! - **编译时优化**：大量优化在编译时完成
//! 
//! ## 扩展指南
//! 
//! ### 自定义方向系统
//! 
//! ```rust
//! use rlwfc::DirectionTrait;
//! 
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
//! enum Direction6 {  // 六角形网格
//!     NorthEast, East, SouthEast, SouthWest, West, NorthWest
//! }
//! 
//! impl DirectionTrait for Direction6 {
//!     // 实现必要的方法...
//! #   fn to_neighbor_index(&self) -> Option<usize> { Some(0) }
//! #   fn opposite(&self) -> Option<Self> { None }
//! #   fn all_directions() -> Vec<Self> { vec![] }
//! #   fn name(&self) -> &'static str { "Custom" }
//! }
//! ```
//! 
//! ### 自定义瓷砖约束
//! 
//! ```rust
//! use rlwfc::{TileSetVirtual, TileId};
//! 
//! struct CustomTileSet {
//!     // 自定义字段...
//! }
//! 
//! impl TileSetVirtual<String> for CustomTileSet {
//!     fn build_tile_set(&mut self) {
//!         // 自定义瓷砖构建逻辑
//!     }
//!     
//!     fn judge_possibility(&self, neighbors: &[Vec<TileId>], candidate: TileId) -> bool {
//!         // 自定义约束判断逻辑
//!         true
//!     }
//! }
//! ```
//! 
//! ## 依赖和兼容性
//! 
//! - **petgraph**: 高性能图数据结构库
//! - **Rust Edition**: 2021及以上
//! - **最低支持版本**: Rust 1.70+
//! 
//! ## 贡献和支持
//! 
//! 欢迎贡献代码、报告问题或提出改进建议。项目遵循Rust社区的行为准则和贡献指南。

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
pub mod tile_set;

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

pub use grid_system::{GridSystem, GridBuilder}; 
pub use tile_set::{TileSetVirtual, TileSet}; 