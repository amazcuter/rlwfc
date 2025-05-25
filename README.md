# RLWFC - Rust Wave Function Collapse Library

🦀 **基于Rust实现的Wave Function Collapse (WFC)算法库**

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![petgraph](https://img.shields.io/badge/petgraph-0.6-blue.svg)](https://crates.io/crates/petgraph)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

## 🌟 特性

- **类型安全**: 使用Rust的类型系统确保内存安全，完全消除空指针解引用
- **方向感知**: 创新的方向识别系统，支持四方向（东南西北）网格操作
- **高性能**: 基于petgraph图库，提供优化的图算法和内存布局
- **零开销抽象**: 使用类型别名和空类型实现零内存开销
- **完备错误处理**: 提供详细的错误类型和处理机制
- **扩展性强**: 支持任意网格拓扑（三角形、六边形、3D等）

## 🚀 快速开始

### 添加依赖

在你的 `Cargo.toml` 中添加：

```toml
[dependencies]
RLWFC = { path = "path/to/RLWFC" }
```

### 基本使用

```rust
use RLWFC::{GridSystem, Cell, Direction4, DirectionTrait};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建网格系统
    let mut grid = GridSystem::new();
    
    // 添加单元格
    let cell1 = grid.add_cell(Cell::with_id(1));
    let cell2 = grid.add_cell(Cell::with_id(2));
    
    // 创建边连接
    grid.create_edge(cell1, cell2)?;
    
    // 获取邻居
    let neighbors = grid.get_neighbors(cell1);
    println!("Cell {:?} has {} neighbors", cell1, neighbors.len());
    
    // 方向查询
    if let Some(east_neighbor) = grid.get_neighbor_by_direction(cell1, Direction4::East) {
        println!("东邻居: {:?}", east_neighbor);
    }
    
    Ok(())
}
```

## 📚 API 文档

### 核心类型

- **`GridSystem`**: 网格系统核心类，提供图操作和方向感知功能
- **`Cell`**: 单元格数据结构
- **`Direction4`**: 四方向枚举（东、南、西、北）
- **`GridError`**: 错误类型定义

### 主要方法

#### GridSystem

```rust
// 创建网格系统
let mut grid = GridSystem::new();

// 添加单元格
let cell_id = grid.add_cell(Cell::new());

// 创建边
grid.create_edge(from_cell, to_cell)?;

// 获取邻居
let neighbors = grid.get_neighbors(cell_id);

// 方向查询
let neighbor = grid.get_neighbor_by_direction(cell_id, Direction4::East);

// 验证结构
grid.validate_structure()?;
```

## 🏗️ 架构设计

### 设计原则

1. **算法库定位**: 专注于提供核心图操作，具体构建逻辑由应用层实现
2. **最小可行设计**: 只包含必要功能，避免过度工程化
3. **方向感知**: 通过有向图和边创建顺序约定实现零开销的方向识别

### 核心创新

#### 方向识别系统

利用petgraph有向图的稳定特性：
- **插入逆序**: `neighbors()`返回边添加的逆序
- **确定性行为**: 边的顺序完全由创建顺序决定
- **零内存开销**: 不需要额外存储方向信息

```rust
// 标准边创建顺序：东向，然后南向
grid.create_edge(center, east)?;   // 第一个边
grid.create_edge(center, south)?;  // 第二个边

// neighbors()返回: [south, east] (逆序)
// Direction4::East  映射到索引 1
// Direction4::South 映射到索引 0
```

## 🧪 测试

运行所有测试：

```bash
cargo test
```

运行示例：

```bash
cargo run --example basic_usage
```

## 📁 项目结构

```
RLWFC/
├── src/
│   ├── lib.rs           # 库入口，重新导出主要类型
│   ├── wfc_util.rs      # 基础类型定义、错误处理、方向系统
│   └── grid_system.rs   # 网格系统实现
├── examples/
│   └── basic_usage.rs   # 基本使用示例
├── Cargo.toml
└── README.md
```

## 🔧 开发

### 构建

```bash
cargo build
```

### 检查代码

```bash
cargo check
```

### 格式化

```bash
cargo fmt
```

### 代码检查

```bash
cargo clippy
```

## 🤝 与原C++代码的对应关系

| C++ | Rust | 说明 |
|-----|------|------|
| `CellID` | `CellId` | 单元格标识符 |
| `EdgeID` | `EdgeId` | 边标识符 |
| `GraphEdge` | `GraphEdge` | 图边数据 |
| `Cell` | `Cell` | 单元格数据 |
| `GridSystem::CreateEdge()` | `GridSystem::create_edge()` | 创建边 |
| `GridSystem::getNeighbor()` | `GridSystem::get_neighbors()` | 获取邻居 |
| `GridSystem::findEdge()` | `GridSystem::find_edge()` | 查找边 |

## 🔮 未来计划

- [ ] 支持更多网格类型（三角形、六边形）
- [ ] 3D网格支持
- [ ] 性能基准测试
- [ ] 更多示例和教程
- [ ] WebAssembly支持

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 👨‍💻 作者

**amazcuter** - amazcuter@outlook.com

## 🙏 致谢

- [petgraph](https://crates.io/crates/petgraph) - 优秀的Rust图库
- Rust社区的支持和贡献 