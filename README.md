# RLWFC - Rust Wave Function Collapse Library

[![Crates.io](https://img.shields.io/crates/v/rlwfc.svg)](https://crates.io/crates/rlwfc)
[![Documentation](https://docs.rs/rlwfc/badge.svg)](https://docs.rs/rlwfc)
[![License](https://img.shields.io/crates/l/rlwfc.svg)](https://github.com/amazcuter/rlwfc#license)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)

RLWFC是一个基于Rust实现的Wave Function Collapse (WFC)算法库，使用petgraph作为底层图数据结构，提供类型安全和高性能的WFC实现。

## 重要更新 - 虚拟边机制

### 解决的关键问题

修复了边界单元格邻居索引错位的严重bug：

- **问题**：边缘单元格缺少某些方向的邻居，导致`neighbors()`返回的索引顺序不一致
- **影响**：在`judge_possibility`中访问邻居时发生数组越界或索引对应错误
- **解决**：引入虚拟边机制，确保所有单元格的邻居索引顺序一致：`[北, 西, 南, 东]`

### 技术实现

```rust
// 新的create_edge API - 支持虚拟边
grid.create_edge(cell, Some(neighbor))?; // 真实邻居
grid.create_edge(cell, None)?;            // 虚拟边（边界处理）

// 虚拟节点检测
if !grid.is_virtual_node(neighbor) {
    // 处理真实邻居
}
```

---

## 项目概述

这是对原C++ WFC系统的完整Rust重写，在保持API兼容性的同时，引入了现代Rust的设计理念和安全保证。

### 核心特性

- **类型安全**：编译时保证类型正确性，避免运行时错误
- **内存安全**：自动内存管理，无悬垂指针和内存泄漏
- **方向感知**：创新的方向识别系统，支持空间方向查询
- **边界处理**：虚拟边机制确保边界单元格的正确处理
- **零成本抽象**：高级抽象不影响运行时性能
- **并发安全**：支持并发访问的数据结构设计

## 安装

在您的 `Cargo.toml` 中添加：

```toml
[dependencies]
rlwfc = "0.1.0"
```

## 快速开始

### 基本使用

```rust
use rlwfc::{GridSystem, Cell, Direction4, GridError};

fn main() -> Result<(), GridError> {
    // 创建网格系统
    let mut grid = GridSystem::new();
    
    // 添加单元格
    let cell1 = grid.add_cell(Cell::with_id(1));
    let cell2 = grid.add_cell(Cell::with_id(2));
    
    // 创建边连接
    grid.create_edge(cell1, Some(cell2))?;
    
    // 方向感知查询
    if let Some(neighbor) = grid.get_neighbor_by_direction(cell1, Direction4::East) {
        println!("Eastern neighbor: {:?}", neighbor);
    }
    
    Ok(())
}
```

### 使用构建器创建复杂网格

```rust
use rlwfc::{GridSystem, GridBuilder, Cell, GridError};

struct LinearGridBuilder {
    size: usize,
}

impl GridBuilder for LinearGridBuilder {
    fn build_grid_system(&mut self, grid: &mut GridSystem) -> Result<(), GridError> {
        let mut cells = Vec::new();
        for i in 0..self.size {
            let cell = grid.add_cell(Cell::with_id(i as u32));
            cells.push(cell);
        }
        
        for i in 0..self.size - 1 {
            grid.create_edge(cells[i], Some(cells[i + 1]))?;
        }
        
        Ok(())
    }
    
    fn get_dimensions(&self) -> Vec<usize> {
        vec![self.size]
    }
    
    fn get_grid_type_name(&self) -> &'static str {
        "Linear"
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let builder = LinearGridBuilder { size: 5 };
    let _grid = GridSystem::from_builder(builder)?;
    Ok(())
}
```

## 模块说明

### wfc_util 模块

提供基础类型定义、错误处理和方向系统：

- 类型别名：`CellId`, `EdgeId`, `TileId` 等
- 数据结构：`Cell`, `GraphEdge`, `Tile` 等
- 方向系统：`DirectionTrait` 和 `Direction4`

### grid_system 模块

实现WFC系统的核心网格管理功能：

- 图操作：基于petgraph的高效图操作
- 方向感知：零成本的方向识别和查询
- 构建器模式：`GridBuilder` trait支持多种网格类型

### tile_set 模块

实现WFC算法的瓷砖管理和约束判断：

- 瓷砖管理：高效的瓷砖存储和查询
- 约束判断：灵活的约束规则实现
- 泛型设计：支持任意类型的边数据

## 示例程序

运行示例程序来了解库的使用：

```bash
# 基本的2D正交WFC示例
cargo run --example orthogonal_2d_wfc

# 基本使用示例
cargo run --example basic_usage

# 网格构建器示例
cargo run --example grid_builder_demo

# 瓷砖系统示例
cargo run --example tile_system_demo
```

## 文档

查看完整的API文档：

- [在线文档](https://docs.rs/rlwfc)
- 本地生成：`cargo doc --open`

文档包含：

- 详细的API参考
- 使用示例和最佳实践
- 架构设计说明
- C++到Rust的映射表

## 与原C++版本对比

| 特性 | C++ | Rust |
|------|-----|------|
| 内存安全 | 手动管理 | 自动保证 |
| 类型安全 | 运行时检查 | 编译时保证 |
| 错误处理 | 异常/返回码 | Result类型 |
| 多态 | 虚函数继承 | Trait组合 |
| 并发 | 手动同步 | 编译时保证 |
| 方向感知 | 无 | 零成本实现 |

## 最低支持的Rust版本 (MSRV)

这个库需要Rust 1.70或更高版本。

## 测试

运行测试套件：

```bash
# 单元测试
cargo test

# 文档测试
cargo test --doc

# 集成测试
cargo test --tests
```

所有核心功能都有对应的单元测试，确保代码质量和正确性。

## 开发状态

✅ 基础类型系统  
✅ 网格系统和图操作  
✅ 方向感知功能  
✅ 瓷砖系统  
✅ WFC管理器  
✅ 构建器模式  
✅ 完整的文档  
✅ 示例程序  
✅ 单元测试  
✅ 集成测试

## 贡献

欢迎贡献代码、报告问题或提出改进建议。请确保：

1. 代码通过所有测试：`cargo test`
2. 代码通过格式检查：`cargo fmt`
3. 代码通过linting：`cargo clippy`
4. 文档是最新的：`cargo doc`

## 许可证

- MIT license ([LICENSE-MIT](LICENSE-MIT) 或 <http://opensource.org/licenses/MIT>)

## 作者

- **amazcuter** - *初始开发* - <amazcuter@outlook.com>

## 致谢

- 感谢petgraph库提供了优秀的图数据结构基础
- 感谢Rust社区提供的工具和最佳实践指导
