//! # RLWFC - Rust版本的波函数塌缩（Wave Function Collapse）算法库
//!
//! 这是一个用Rust实现的波函数塌缩算法库，从C++版本迁移而来。
//! 使用petgraph作为底层图结构，提供高性能和类型安全的WFC实现。
//!
//! ## 核心特性
//!
//! - **类型安全**：使用Rust类型系统确保编译时安全
//! - **高性能**：基于petgraph的优化图操作
//! - **可扩展**：支持不同类型的网格和约束
//! - **内存安全**：利用Rust的所有权系统避免内存错误
//!
//! ## ⚠️ 重要设计约束：边创建顺序
//!
//! 本库的核心设计基于一个关键约束：**边创建的全局一致顺序**。
//!
//! ### 为什么这很重要？
//!
//! - **方向识别**：WFC算法需要识别邻居的相对方向
//! - **无向连接**：WFC本质上需要无向连接，但我们使用有向图实现
//! - **技术实现**：通过petgraph的neighbor返回顺序实现方向识别
//!
//! ### 基本使用原则
//!
//! ```rust,no_run
//! use rlwfc::{GridSystem, GridBuilder, Direction4, GridError};
//!
//! // ✅ 正确：在GridBuilder实现中按固定顺序创建边
//! struct My2DGrid;
//! impl GridBuilder for My2DGrid {
//!     fn build_grid_system(&mut self, grid: &mut GridSystem) -> Result<(), GridError> {
//!         // 为每个单元格按相同顺序创建边：东、南、西、北
//!         // 这样neighbors()返回 [北, 西, 南, 东] (逆序)
//!         # Ok(())
//!     }
//!     # fn get_dimensions(&self) -> Vec<usize> { vec![2, 2] }
//!     # fn get_grid_type_name(&self) -> &'static str { "My2D" }
//! }
//!
//! // ❌ 错误：直接调用create_edge而不考虑顺序
//! let mut grid = GridSystem::new();
//! // 这样做会破坏方向识别系统
//! ```
//!
//! ### 为什么不提供便捷方法？
//!
//! 本库故意**不提供**诸如`create_undirected_connection()`的便捷方法，因为：
//!
//! 1. **顺序破坏**：自动双向连接可能破坏全局边创建顺序
//! 2. **应用层责任**：正确的网格构建逻辑应由具体的GridBuilder实现
//! 3. **错误预防**：通过设计约束防止错误使用
//!
//! ### 瓷砖边数据顺序约定
//!
//! 同样重要的是，**瓷砖的边数据必须严格按照相同的顺序排列**：
//!
//! ```rust,no_run
//! use rlwfc::TileSet;
//!
//! let mut tile_set = TileSet::new();
//!
//! // ✅ 正确：瓷砖边数据按 [北, 西, 南, 东] 顺序排列
//! tile_set.add_tile(vec![
//!     "forest",  // 北边数据 (索引 0)
//!     "water",   // 西边数据 (索引 1)  
//!     "grass",   // 南边数据 (索引 2)
//!     "stone",   // 东边数据 (索引 3)
//! ], 10);
//!
//! // ❌ 错误：任意顺序会破坏兼容性检查
//! tile_set.add_tile(vec!["stone", "forest", "water", "grass"], 5);
//! ```
//!
//! 这确保了：
//! - 直接索引对应：`tile.edges[i]` 对应 `neighbors()[i]` 的方向
//! - 高效兼容性检查：无需运行时映射转换
//! - 统一的系统约定：网格和瓷砖使用相同的索引语义
//!
//! ## 基本使用示例
//!
//! ```rust,no_run
//! use rlwfc::{GridSystem, Cell, TileSet, Direction4, GridError};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 1. 创建网格系统
//!     let mut grid = GridSystem::new();
//!     
//!     // 2. 添加单元格
//!     let cell1 = grid.add_cell(Cell::with_id(1));
//!     let cell2 = grid.add_cell(Cell::with_id(2));
//!     
//!     // 3. 创建连接
//!     grid.create_edge(cell1, Some(cell2))?;
//!     
//!     // 4. 方向查询
//!     if let Some(neighbor) = grid.get_neighbor_by_direction(cell1, Direction4::East) {
//!         println!("Found eastern neighbor: {:?}", neighbor);
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## 模块结构
//!
//! - [`grid_system`] - 网格管理和图结构操作
//! - [`wfc_util`] - WFC算法核心实现  
//! - [`tile_set`] - 瓦片管理和兼容性规则
//! - [`Cell`] - 单元格数据结构
//! - [`Tile`] - 瓦片数据结构
//!
//! ## 设计哲学
//!
//! 1. **最小可行设计**：只包含必要功能，避免过度工程化
//! 2. **应用层控制**：具体的网格构建逻辑由应用层决定
//! 3. **类型安全**：利用Rust类型系统防止运行时错误
//! 4. **可组合性**：支持不同组件的灵活组合
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
//! use rlwfc::{GridSystem, Cell, Direction4, GridError};
//!
//! fn main() -> Result<(), GridError> {
//!     // 创建网格系统
//!     let mut grid = GridSystem::new();
//!     
//!     // 添加单元格
//!     let cell1 = grid.add_cell(Cell::with_id(1));
//!     let cell2 = grid.add_cell(Cell::with_id(2));
//!     
//!     // 创建边连接
//!     grid.create_edge(cell1, Some(cell2))?;
//!     
//!     // 获取邻居
//!     let neighbors = grid.get_neighbors(cell1);
//!     println!("Cell {:?} has {} neighbors", cell1, neighbors.len());
//!     
//!     // 方向感知查询
//!     if let Some(eastern_neighbor) = grid.get_neighbor_by_direction(cell1, Direction4::East) {
//!         println!("Eastern neighbor: {:?}", eastern_neighbor);
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### 使用构建器创建复杂网格
//!
//! ```rust,no_run
//! use rlwfc::{GridSystem, GridBuilder, Cell, GridError};
//!
//! struct LinearGridBuilder {
//!     size: usize,
//! }
//!
//! impl GridBuilder for LinearGridBuilder {
//!     fn build_grid_system(&mut self, grid: &mut GridSystem) -> Result<(), GridError> {
//!         let mut cells = Vec::new();
//!         for i in 0..self.size {
//!             let cell = grid.add_cell(Cell::with_id(i as u32));
//!             cells.push(cell);
//!         }
//!         
//!         for i in 0..self.size - 1 {
//!             grid.create_edge(cells[i], Some(cells[i + 1]))?;
//!         }
//!         
//!         Ok(())
//!     }
//!     
//!     fn get_dimensions(&self) -> Vec<usize> {
//!         vec![self.size]
//!     }
//!     
//!     fn get_grid_type_name(&self) -> &'static str {
//!         "Linear"
//!     }
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 使用示例
//!     let builder = LinearGridBuilder { size: 5 };
//!     let _grid = GridSystem::from_builder(builder)?;
//!     
//!     Ok(())
//! }
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
//! use rlwfc::{TileSetVirtual, TileId, Tile, GridError};
//!
//! struct CustomTileSet {
//!     // 自定义字段...
//! }
//!
//! impl TileSetVirtual<String> for CustomTileSet {
//!     fn build_tile_set(&mut self) -> Result<(), GridError> {
//!         // 自定义瓷砖构建逻辑
//!         Ok(())
//!     }
//!     
//!     fn judge_possibility(&self, _neighbors: &[Vec<TileId>], _candidate: TileId) -> bool {
//!         // 自定义约束判断逻辑
//!         true
//!     }
//!     
//!     fn get_tile(&self, _id: TileId) -> Option<&Tile<String>> {
//!         None
//!     }
//!     
//!     fn get_tile_count(&self) -> usize {
//!         0
//!     }
//!     
//!     fn get_all_tile_ids(&self) -> Vec<TileId> {
//!         vec![]
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

pub mod grid_system;
pub mod tile_set;
pub mod wfc_manager;
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

// 重新导出主要类型，方便使用
pub use wfc_util::{
    // 工具函数
    find_in_2d_vector,
    // 数据结构
    Cell,
    // 基础类型
    CellId,
    Cells,
    Direction4,

    // 方向系统
    DirectionTrait,
    EdgeId,
    Edges,
    GraphEdge,
    // 错误处理
    GridError,

    Tile,

    TileId,
    Tiles,
    WFCGraph,
};

pub use grid_system::{GridBuilder, GridSystem};
pub use tile_set::{TileSet, TileSetVirtual};
pub use wfc_manager::{
    CellState, CellWfcData, DefaultInitializer, StepResult, WfcConfig, WfcError, WfcInitializer,
    WfcManager,
};
