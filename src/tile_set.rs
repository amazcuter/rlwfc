//! # 瓷砖集模块
//!
//! 本模块实现了WFC（Wave Function Collapse）系统的瓷砖集功能，
//! 是对原C++ TileSet.h的Rust重写版本。
//!
//! ## 模块概述
//!
//! 瓷砖集是WFC算法的核心组件之一，负责：
//!
//! - **瓷砖管理**：存储和管理所有可用的瓷砖
//! - **约束判断**：判断瓷砖在特定邻居约束下的可能性
//! - **瓷砖构建**：初始化和配置瓷砖集合
//!
//! ## 设计架构
//!
//! ### 分离关注点设计
//!
//! 与原C++的单一类设计不同，Rust版本采用了分离关注点的架构：
//!
//! - [`TileSetVirtual`] trait：仅包含原C++的两个虚函数
//! - [`TileSet`] struct：包含所有固定方法和数据存储
//!
//! 这种设计的优势：
//!
//! 1. **清晰的职责划分**：虚函数逻辑与数据管理分离
//! 2. **更好的组合性**：可以独立实现和测试两部分
//! 3. **避免继承复杂性**：使用组合替代继承
//!
//! ### 与原C++的对比
//!
//! | 方面 | C++ | Rust |
//! |------|-----|------|
//! | 多态机制 | 虚函数继承 | Trait + 组合 |
//! | 内存管理 | 手动管理 | 自动管理 |
//! | 类型安全 | 运行时检查 | 编译时检查 |
//! | 错误处理 | 异常/返回码 | Result类型 |
//! | 泛型支持 | 模板 | 泛型 + trait约束 |
//!
//! ## 使用模式
//!
//! ### 基本用法
//!
//! ```rust
//! use rlwfc::{TileSet, Tile};
//!
//! // 创建瓷砖集
//! let mut tile_set = TileSet::new();
//!
//! // ⚠️ 重要：边数据必须按 neighbors() 返回顺序排列
//! let tile_edges = vec![
//!     "北边数据",  // 索引 0 - 对应 neighbors()[0] (北方向)
//!     "西边数据",  // 索引 1 - 对应 neighbors()[1] (西方向)  
//!     "南边数据",  // 索引 2 - 对应 neighbors()[2] (南方向)
//!     "东边数据",  // 索引 3 - 对应 neighbors()[3] (东方向)
//! ];
//! let tile_id = tile_set.add_tile(tile_edges, 10);
//!
//! // 获取瓷砖
//! if let Some(tile) = tile_set.get_tile(tile_id) {
//!     println!("Tile weight: {}", tile.weight);
//! }
//! ```
//!
//! ### ⚠️ **关键设计约束：瓷砖边数据顺序**
//!
//! 瓷砖的边数据顺序必须与网格系统的 `neighbors()` 返回顺序**严格一致**：
//!
//! #### 顺序对应关系
//!
//! ```text
//! 网格边创建顺序：东 → 南 → 西 → 北
//! neighbors() 返回：[北, 西, 南, 东] (petgraph 逆序特性)
//! 瓷砖边数据索引：[0,  1,  2,  3]
//! 方向到索引映射：北=0, 西=1, 南=2, 东=3
//! ```
//!
//! #### 为什么这很重要？
//!
//! 1. **直接索引对应**：`judge_possibility()` 中可以直接用索引访问对应方向的边数据
//! 2. **高效兼容性检查**：无需额外的方向映射转换
//! 3. **统一约定**：网格系统和瓷砖系统使用相同的索引语义
//! 4. **零成本抽象**：运行时无额外开销
//!
//! #### 正确的瓷砖创建模式
//!
//! ```rust
//! use rlwfc::TileSet;
//!
//! let mut tile_set = TileSet::new();
//!
//! // ✅ 正确：按 neighbors() 顺序排列边数据
//! tile_set.add_tile(vec![
//!     "grass",  // 北边 (索引 0)
//!     "water",  // 西边 (索引 1)
//!     "grass",  // 南边 (索引 2)  
//!     "water",  // 东边 (索引 3)
//! ], 10);
//!
//! // ❌ 错误：随意排列会破坏方向对应关系
//! tile_set.add_tile(vec![
//!     "water",  // 这样的顺序无法与 neighbors() 正确对应
//!     "grass",
//!     "water",
//!     "grass",
//! ], 10);
//! ```
//!
//! #### 在 judge_possibility() 中的应用
//!
//! ```rust
//! use rlwfc::TileId;
//!
//! fn judge_possibility(
//!     neighbor_possibilities: &[Vec<TileId>],
//!     candidate: TileId
//! ) -> bool {
//!     // 示例实现
//!     !neighbor_possibilities.is_empty() && candidate < 100
//! }
//! ```
//!
//! ### 实现虚函数trait
//!
//! ```rust
//! use rlwfc::{TileSetVirtual, TileSet, TileId, Tile, GridError};
//!
//! struct MyTileSet {
//!     tiles: TileSet<String>,
//! }
//!
//! impl TileSetVirtual<String> for MyTileSet {
//!     fn build_tile_set(&mut self) -> Result<(), GridError> {
//!         // 清空现有瓷砖
//!         self.tiles.clear();
//!         
//!         // 添加具体的瓷砖
//!         self.tiles.add_tile(vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string()], 10);
//!         self.tiles.add_tile(vec!["B".to_string(), "A".to_string(), "D".to_string(), "C".to_string()], 15);
//!         Ok(())
//!     }
//!     
//!     fn judge_possibility(
//!         &self,
//!         neighbor_possibilities: &[Vec<TileId>],
//!         candidate: TileId
//!     ) -> bool {
//!         // 实现具体的约束判断逻辑
//!         if let Some(_tile) = self.tiles.get_tile(candidate) {
//!             // 检查候选瓷砖是否与邻居兼容
//!             // 这里应该实现具体的兼容性检查逻辑
//!             !neighbor_possibilities.is_empty()
//!         } else {
//!             false
//!         }
//!     }
//!     
//!     fn get_tile(&self, tile_id: TileId) -> Option<&Tile<String>> {
//!         self.tiles.get_tile(tile_id)
//!     }
//!     
//!     fn get_tile_count(&self) -> usize {
//!         self.tiles.get_tile_count()
//!     }
//!     
//!     fn get_all_tile_ids(&self) -> Vec<TileId> {
//!         self.tiles.get_all_tile_ids()
//!     }
//! }
//! ```
//!
//! ## 泛型设计
//!
//! 瓷砖系统支持任意类型的边数据：
//!
//! ```rust
//! use rlwfc::TileSet;
//!
//! // 字符串边数据
//! let mut string_tiles = TileSet::<String>::new();
//!
//! // 数字边数据  
//! let mut number_tiles = TileSet::<i32>::new();
//!
//! // 自定义结构体边数据
//! #[derive(Clone, PartialEq, Debug)]
//! struct CustomEdge { id: u32, color: String }
//! let mut custom_tiles = TileSet::<CustomEdge>::new();
//! ```
//!
//! ## 性能考虑
//!
//! - **零成本抽象**：trait分发在编译时确定
//! - **内存效率**：紧凑的数据布局，最小化内存占用
//! - **缓存友好**：瓷砖数据连续存储，提高访问效率
//!
//! ## 扩展性
//!
//! 系统设计支持多种扩展：
//!
//! - **不同约束规则**：通过实现`TileSetVirtual`支持各种约束逻辑
//! - **多种边数据类型**：泛型设计支持任意边数据
//! - **性能优化**：可以在具体实现中添加缓存、索引等优化

/**
 * @file tile_set.rs
 * @author amazcuter (amazcuter@outlook.com)
 * @brief WFC系统瓷砖集 - Rust重写版本
 *        对应原C++ TileSet.h的功能，使用trait替代虚函数
 * @version 1.0
 * @date 2025-01-25
 *
 * @copyright Copyright (c) 2025
 */
use crate::wfc_util::*;

// =============================================================================
// 虚函数特性 - 仅包含原C++的两个虚函数
// =============================================================================

/// 瓷砖集虚函数特性 - 仅包含C++的两个虚函数
///
/// 这个trait专门提取了原C++代码中的两个纯虚函数，实现了与原C++设计的完全对应：
///
/// - `virtual void buildTileSet() = 0;`
/// - `virtual bool judgePossibility(...) = 0;`
///
/// ## 设计理念
///
/// ### 职责分离
///
/// 将虚函数逻辑从数据管理中分离出来，带来以下好处：
///
/// 1. **清晰的接口**：只包含需要自定义实现的方法
/// 2. **类型安全**：编译时确保所有必要方法都被实现
/// 3. **测试友好**：可以独立模拟和测试虚函数逻辑
///
/// ### 与原C++的一致性
///
/// | C++虚函数 | Rust trait方法 | 功能 |
/// |-----------|----------------|------|
/// | `buildTileSet()` | [`build_tile_set()`] | 构建瓷砖集 |
/// | `judgePossibility(...)` | [`judge_possibility(...)`] | 判断瓷砖可能性 |
///
/// ## 泛型参数
///
/// `EdgeData` 类型参数表示瓷砖边的数据类型，需要满足：
///
/// - `Clone`：支持复制操作
/// - `PartialEq`：支持相等性比较
/// - `Debug`：支持调试输出
///
/// ## 实现示例
///
/// ```rust,no_run
/// use rlwfc::{TileSetVirtual, TileSet, TileId, Tile, GridError};
///
/// struct SimpleTileSet {
///     tiles: TileSet<&'static str>,
/// }
///
/// impl TileSetVirtual<&'static str> for SimpleTileSet {
///     fn build_tile_set(&mut self) -> Result<(), GridError> {
///         // 清空现有瓷砖
///         self.tiles.clear();
///         
///         // 添加具体的瓷砖
///         self.tiles.add_tile(vec!["A", "B", "C", "D"], 10);
///         self.tiles.add_tile(vec!["B", "A", "D", "C"], 15);
///         Ok(())
///     }
///
///     fn judge_possibility(
///         &self,
///         neighbor_possibilities: &[Vec<TileId>],
///         candidate: TileId
///     ) -> bool {
///         // 实现具体的约束判断逻辑
///         if let Some(_tile) = self.tiles.get_tile(candidate) {
///             // 检查候选瓷砖是否与邻居兼容
///             // 这里应该实现具体的兼容性检查逻辑
///             !neighbor_possibilities.is_empty()
///         } else {
///             false
///         }
///     }
///     
///     fn get_tile(&self, tile_id: TileId) -> Option<&Tile<&'static str>> {
///         self.tiles.get_tile(tile_id)
///     }
///     
///     fn get_tile_count(&self) -> usize {
///         self.tiles.get_tile_count()
///     }
///     
///     fn get_all_tile_ids(&self) -> Vec<TileId> {
///         self.tiles.get_all_tile_ids()
///     }
/// }
/// ```
///
/// [`build_tile_set()`]: TileSetVirtual::build_tile_set
/// [`judge_possibility(...)`]: TileSetVirtual::judge_possibility
pub trait TileSetVirtual<EdgeData>
where
    EdgeData: Clone + PartialEq + std::fmt::Debug,
{
    /// 构建瓷砖集 - 对应C++的buildTileSet()虚函数
    ///
    /// 这个方法负责初始化和填充瓷砖集合。具体的实现由各种不同的瓷砖集类型决定。
    ///
    /// ## 实现要求
    ///
    /// 实现者应该在此方法中：
    ///
    /// 1. **清理现有状态**：清空或重置瓷砖集合
    /// 2. **创建瓷砖**：添加所有需要的瓷砖到集合中
    /// 3. **设置属性**：配置每个瓷砖的权重、边数据等
    /// 4. **验证完整性**：确保瓷砖集合的一致性和完整性
    ///
    /// ## 调用时机
    ///
    /// 这个方法通常在以下时机被调用：
    ///
    /// - WFC系统初始化时
    /// - 重新开始新的生成过程时
    /// - 动态改变瓷砖集配置时
    ///
    /// ## 示例实现
    ///
    /// ```rust,no_run
    /// # use rlwfc::TileSet;
    /// # struct MySelf { tiles: TileSet<&'static str> }
    /// # impl MySelf {
    /// fn build_tile_set(&mut self) -> Result<(), rlwfc::GridError> {
    ///     // 1. 清理现有瓷砖
    ///     self.tiles.clear();
    ///     
    ///     // 2. 添加基础瓷砖
    ///     self.tiles.add_tile(vec!["grass", "grass", "grass", "grass"], 50);
    ///     self.tiles.add_tile(vec!["water", "water", "water", "water"], 30);
    ///     
    ///     // 3. 添加过渡瓷砖
    ///     self.tiles.add_tile(vec!["grass", "water", "grass", "water"], 20);
    ///     
    ///     // 4. 可选：添加验证逻辑
    ///     debug_assert!(!self.tiles.is_empty());
    ///     Ok(())
    /// }
    /// # }
    /// ```
    fn build_tile_set(&mut self) -> Result<(), GridError>;

    /// 判断瓷砖可能性 - 对应C++的judgePossibility()虚函数
    ///
    /// 这是WFC算法的核心约束判断方法。它决定了在给定邻居约束的情况下，
    /// 某个候选瓷砖是否可以放置在当前位置。
    ///
    /// # ⚠️ 重要：利用边数据顺序约定
    ///
    /// 由于瓷砖的边数据严格按照 `neighbors()` 返回顺序排列，
    /// 本方法可以直接通过索引访问对应方向的边数据，实现高效的兼容性检查。
    ///
    /// ## 索引到方向的直接映射
    ///
    /// ```text
    /// neighbor_possibilities[0] <-> candidate_tile.edges[0] (北方向)
    /// neighbor_possibilities[1] <-> candidate_tile.edges[1] (西方向)
    /// neighbor_possibilities[2] <-> candidate_tile.edges[2] (南方向)
    /// neighbor_possibilities[3] <-> candidate_tile.edges[3] (东方向)
    /// ```
    ///
    /// ## 高效实现模式
    ///
    /// ```rust,no_run
    /// # use rlwfc::TileId;
    /// # struct MySelf;
    /// # impl MySelf { fn get_tile(&self, id: TileId) -> Option<&rlwfc::Tile<&str>> { None } }
    /// # impl MySelf {
    /// fn judge_possibility(
    ///     &self,
    ///     neighbor_possibilities: &[Vec<TileId>],
    ///     candidate: TileId
    /// ) -> bool {
    ///     let Some(candidate_tile) = self.get_tile(candidate) else {
    ///         return false;
    ///     };
    ///     
    ///     for (direction_index, neighbor_tiles) in neighbor_possibilities.iter().enumerate() {
    ///         // 🎯 直接获取候选瓷砖在该方向的边数据
    ///         let candidate_edge = &candidate_tile.edges[direction_index];
    ///         
    ///         // 检查与该方向所有可能邻居的兼容性
    ///         let is_compatible = neighbor_tiles.iter().any(|&neighbor_id| {
    ///             if let Some(neighbor_tile) = self.get_tile(neighbor_id) {
    ///                 // 获取邻居瓷砖相对方向的边数据
    ///                 let opposite_index = match direction_index {
    ///                     0 => 2,  // 北 ↔ 南
    ///                     1 => 3,  // 西 ↔ 东
    ///                     2 => 0,  // 南 ↔ 北  
    ///                     3 => 1,  // 东 ↔ 西
    ///                     _ => return false,
    ///                 };
    ///                 let neighbor_edge = &neighbor_tile.edges[opposite_index];
    ///                 
    ///                 // 边兼容性检查（具体规则由应用定义）
    ///                 candidate_edge == neighbor_edge
    ///             } else {
    ///                 false
    ///             }
    ///         });
    ///         
    ///         if !is_compatible {
    ///             return false;
    ///         }
    ///     }
    ///     true
    /// }
    /// # }
    /// ```
    ///
    /// ## 性能优势
    ///
    /// 通过边数据顺序约定，该方法获得了显著的性能优势：
    ///
    /// 1. **零成本索引映射**：无需运行时的方向转换或查找表
    /// 2. **O(1) 边数据访问**：直接数组索引，最高效的访问方式
    /// 3. **缓存友好**：连续的内存访问模式，提高CPU缓存命中率
    /// 4. **编译时优化**：编译器可以更好地优化索引访问代码
    ///
    /// # 参数
    ///
    /// * `neighbor_possibilities` - 邻居单元格的可能瓷砖列表数组
    ///   - 每个元素是一个邻居的可能瓷砖ID列表
    ///   - 数组的顺序对应方向顺序：[北, 西, 南, 东]
    ///   - 空列表表示该方向没有邻居或邻居未确定
    ///
    /// * `candidate` - 候选瓷砖的ID
    ///
    /// # 返回值
    ///
    /// * `true` - 该瓷砖在当前邻居约束下是可能的
    /// * `false` - 该瓷砖与邻居约束冲突，不能放置
    ///
    /// # 错误情况
    ///
    /// 实现者应该处理以下错误情况：
    ///
    /// - 候选瓷砖ID无效（不存在对应的瓷砖）
    /// - 邻居瓷砖ID无效
    /// - 边数据索引越界（瓷砖边数量不足）
    ///
    /// ## 算法逻辑
    ///
    /// 典型的实现流程：
    ///
    /// 1. **验证候选瓷砖**：确认候选瓷砖存在且有效
    /// 2. **遍历方向约束**：检查每个方向的邻居约束
    /// 3. **获取边数据**：直接通过索引获取对应方向的边数据
    /// 4. **兼容性检查**：验证候选瓷砖的边与邻居瓷砖的边是否兼容
    /// 5. **返回结果**：所有方向都兼容则返回true，否则返回false
    ///
    /// ## 性能考虑
    ///
    /// 这个方法在WFC算法中会被频繁调用，因此性能很重要：
    ///
    /// - 考虑缓存计算结果
    /// - 优先检查最容易失败的约束
    /// - 使用快速的边比较算法
    /// - 利用边数据顺序约定避免额外的映射开销
    fn judge_possibility(&self, neighbor_possibilities: &[Vec<TileId>], candidate: TileId) -> bool;

    /// 获取指定ID的瓷砖
    fn get_tile(&self, tile_id: TileId) -> Option<&Tile<EdgeData>>;

    /// 获取瓷砖总数
    fn get_tile_count(&self) -> usize;

    /// 获取所有瓷砖ID列表
    fn get_all_tile_ids(&self) -> Vec<TileId>;
}

// =============================================================================
// 瓷砖集具体实现 - 包含所有固定方法和数据存储
// =============================================================================

/// 瓷砖集具体实现 - 包含所有固定方法和数据存储
#[derive(Debug, Clone)]
pub struct TileSet<EdgeData>
where
    EdgeData: Clone + PartialEq + std::fmt::Debug,
{
    /// 瓷砖列表 - 对应C++的tiles_成员
    tiles: Vec<Tile<EdgeData>>,
}

impl<EdgeData> TileSet<EdgeData>
where
    EdgeData: Clone + PartialEq + std::fmt::Debug,
{
    /// 创建新的瓷砖集
    pub fn new() -> Self {
        Self { tiles: Vec::new() }
    }

    /// 添加瓷砖 - 对应C++的addTile方法
    ///
    /// # ⚠️ 重要：边数据顺序约束
    ///
    /// 传入的 `edges` 向量必须严格按照 `neighbors()` 返回顺序排列，
    /// 即：**[北, 西, 南, 东]** 的顺序。
    ///
    /// ## 顺序约定的重要性
    ///
    /// 这个顺序约定确保了：
    ///
    /// 1. **直接索引映射**：`judge_possibility()` 中可以直接通过索引访问对应方向的边数据
    /// 2. **零成本抽象**：无需运行时的方向转换
    /// 3. **统一语义**：网格系统和瓷砖系统使用相同的索引语义
    /// 4. **高效兼容性检查**：O(1) 时间复杂度的边数据访问
    ///
    /// ## 索引到方向的映射
    ///
    /// ```text
    /// edges[0] -> 北方向的边数据 (对应 neighbors()[0])
    /// edges[1] -> 西方向的边数据 (对应 neighbors()[1])  
    /// edges[2] -> 南方向的边数据 (对应 neighbors()[2])
    /// edges[3] -> 东方向的边数据 (对应 neighbors()[3])
    /// ```
    ///
    /// ## 正确使用示例
    ///
    /// ```rust
    /// use rlwfc::TileSet;
    ///
    /// let mut tile_set = TileSet::new();
    ///
    /// // ✅ 正确：按照 [北, 西, 南, 东] 顺序排列
    /// tile_set.add_tile(vec![
    ///     "forest",  // 北边：与北邻居连接的边
    ///     "water",   // 西边：与西邻居连接的边
    ///     "grass",   // 南边：与南邻居连接的边
    ///     "stone",   // 东边：与东邻居连接的边
    /// ], 10);
    ///
    /// // ❌ 错误：任意顺序会破坏方向对应关系
    /// tile_set.add_tile(vec![
    ///     "stone",   // 这样排列无法正确对应方向
    ///     "grass",
    ///     "water",
    ///     "forest",
    /// ], 5);
    /// ```
    ///
    /// ## 在兼容性判断中的应用
    ///
    /// 正确的边数据顺序使得兼容性判断变得高效：
    ///
    /// ```rust,no_run
    /// # use rlwfc::{TileSetVirtual, TileId};
    /// # struct MySelf;
    /// # impl MySelf { fn get_tile(&self, id: TileId) -> Option<&rlwfc::Tile<&str>> { None } }
    /// # impl MySelf {
    /// fn judge_possibility(
    ///     &self,
    ///     neighbor_possibilities: &[Vec<TileId>],
    ///     candidate: TileId
    /// ) -> bool {
    ///     let Some(candidate_tile) = self.get_tile(candidate) else {
    ///         return false;
    ///     };
    ///     
    ///     for (direction_index, neighbor_tiles) in neighbor_possibilities.iter().enumerate() {
    ///         // 🎯 直接获取候选瓷砖在该方向的边数据
    ///         let candidate_edge = &candidate_tile.edges[direction_index];
    ///         
    ///         // 检查与该方向所有可能邻居的兼容性
    ///         let is_compatible = neighbor_tiles.iter().any(|&neighbor_id| {
    ///             if let Some(neighbor_tile) = self.get_tile(neighbor_id) {
    ///                 // 获取邻居瓷砖相对方向的边数据
    ///                 let opposite_index = match direction_index {
    ///                     0 => 2,  // 北 ↔ 南
    ///                     1 => 3,  // 西 ↔ 东
    ///                     2 => 0,  // 南 ↔ 北  
    ///                     3 => 1,  // 东 ↔ 西
    ///                     _ => return false,
    ///                 };
    ///                 let neighbor_edge = &neighbor_tile.edges[opposite_index];
    ///                 
    ///                 // 边兼容性检查（具体规则由应用定义）
    ///                 candidate_edge == neighbor_edge
    ///             } else {
    ///                 false
    ///             }
    ///         });
    ///         
    ///         if !is_compatible {
    ///             return false;
    ///         }
    ///     }
    ///     true
    /// }
    /// # }
    /// ```
    ///
    /// # 参数
    ///
    /// * `edges` - 边数据列表，必须按 [北, 西, 南, 东] 顺序排列
    /// * `weight` - 瓷砖权重，影响在WFC算法中被选择的概率
    ///
    /// # 返回值
    ///
    /// * 新创建瓷砖的ID，可用于后续的瓷砖引用和查询
    ///
    /// # 性能说明
    ///
    /// - 时间复杂度：O(1) - 直接向量追加
    /// - 空间复杂度：O(E) - E为边数据的大小
    /// - 瓷砖ID就是其在内部向量中的索引，查询效率为O(1)
    pub fn add_tile(&mut self, edges: Vec<EdgeData>, weight: i32) -> TileId {
        let tile_id = self.tiles.len();
        let tile = Tile::new(tile_id, weight, edges);
        self.tiles.push(tile);
        tile_id
    }

    /// 获取所有瓷砖 - 对应C++的getAllTiles()方法
    pub fn get_all_tiles(&self) -> &[Tile<EdgeData>] {
        &self.tiles
    }

    /// 获取所有瓷砖ID
    pub fn get_all_tile_ids(&self) -> Vec<TileId> {
        (0..self.tiles.len()).collect()
    }

    /// 根据ID获取瓷砖
    pub fn get_tile(&self, tile_id: TileId) -> Option<&Tile<EdgeData>> {
        self.tiles.get(tile_id)
    }

    /// 获取瓷砖数量
    pub fn get_tile_count(&self) -> usize {
        self.tiles.len()
    }

    /// 清空瓷砖集
    pub fn clear(&mut self) {
        self.tiles.clear();
    }

    /// 检查瓷砖是否存在
    pub fn contains_tile(&self, tile_id: TileId) -> bool {
        tile_id < self.tiles.len()
    }

    /// 检查瓷砖集是否为空
    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }
}

impl<EdgeData> Default for TileSet<EdgeData>
where
    EdgeData: Clone + PartialEq + std::fmt::Debug,
{
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

    // 测试用的简单瓷砖集实现
    struct TestTileSet {
        tiles: TileSet<&'static str>,
    }

    impl TestTileSet {
        pub fn new() -> Self {
            Self {
                tiles: TileSet::new(),
            }
        }
    }

    impl TileSetVirtual<&'static str> for TestTileSet {
        fn build_tile_set(&mut self) -> Result<(), GridError> {
            // 构建简单的测试瓷砖集
            self.tiles.clear();
            self.tiles.add_tile(vec!["A", "A", "A", "A"], 10);
            self.tiles.add_tile(vec!["B", "B", "B", "B"], 10);
            self.tiles.add_tile(vec!["A", "B", "A", "B"], 5);
            Ok(())
        }

        fn judge_possibility(
            &self,
            _neighbor_possibilities: &[Vec<TileId>],
            candidate: TileId,
        ) -> bool {
            // 检查候选瓷砖是否存在
            if self.tiles.get_tile(candidate).is_none() {
                return false;
            }

            // 简单测试实现，存在的瓷砖都兼容
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
    fn test_tile_set_creation() {
        let tile_set = TileSet::<&str>::new();
        assert_eq!(tile_set.get_tile_count(), 0);
        assert!(tile_set.is_empty());
    }

    #[test]
    fn test_add_and_get_tiles() {
        let mut tile_set = TileSet::new();

        // 添加瓷砖
        let tile_id1 = tile_set.add_tile(vec!["A", "B", "C", "D"], 10);
        let tile_id2 = tile_set.add_tile(vec!["B", "A", "D", "C"], 15);

        assert_eq!(tile_id1, 0);
        assert_eq!(tile_id2, 1);
        assert_eq!(tile_set.get_tile_count(), 2);

        // 获取瓷砖
        let tile1 = tile_set.get_tile(tile_id1).unwrap();
        assert_eq!(tile1.weight, 10);
        assert_eq!(tile1.edges, vec!["A", "B", "C", "D"]);
    }

    #[test]
    fn test_tile_set_virtual_implementation() {
        let mut test_tile_set = TestTileSet::new();

        // 测试构建瓷砖集
        test_tile_set.build_tile_set().unwrap();
        assert_eq!(test_tile_set.get_tile_count(), 3);

        // 测试判断可能性
        let neighbor_possibilities = vec![vec![0, 1], vec![1, 2]];
        let is_possible = test_tile_set.judge_possibility(&neighbor_possibilities, 0);
        assert!(is_possible);

        // 测试不存在的瓷砖
        let is_possible = test_tile_set.judge_possibility(&neighbor_possibilities, 10);
        assert!(!is_possible);
    }
}
