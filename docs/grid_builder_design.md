# GridBuilder Trait 设计文档

## 概述

`GridBuilder` trait 是对原C++代码中 `buildGridSystem()` 纯虚函数的Rust实现，提供了一种类型安全、可组合的方式来构建不同类型的网格系统。

## 原C++设计回顾

在原C++代码中：

```cpp
class GridSystem {
    // 建立网格系统纯虚函数
    virtual void buildGridSystem() = 0;
    
    // 其他方法...
};

class Orthogonal2DGrid : public GridSystem {
    virtual void buildGridSystem() override {
        // 具体的2D正交网格构建逻辑
    }
};
```

这种设计的优点：
- 抽象了网格构建逻辑
- 支持多种网格类型（2D正交、六角形、3D等）
- 通过继承实现多态

## Rust实现的改进

### 1. Trait设计

```rust
pub trait GridBuilder {
    /// 构建网格系统，对应原C++的buildGridSystem()纯虚函数
    fn build_grid_system(&mut self, grid: &mut GridSystem) -> Result<(), GridError>;
    
    /// 获取网格的维度信息（可选实现）
    fn get_dimensions(&self) -> Vec<usize> {
        vec![]
    }
    
    /// 获取网格类型的名称（可选实现）
    fn get_grid_type_name(&self) -> &'static str {
        "CustomGrid"
    }
}
```

### 2. 使用方式

#### 方式一：分离构建
```rust
let mut grid = GridSystem::new();
let builder = Orthogonal2DBuilder::new(width, height);
grid.build_with(builder)?;
```

#### 方式二：直接构建
```rust
let builder = Orthogonal2DBuilder::new(width, height);
let grid = GridSystem::from_builder(builder)?;
```

## 相比C++的优势

### 1. 类型安全
- 编译时错误检查
- 没有运行时类型转换
- 明确的错误处理（Result类型）

### 2. 组合而非继承
- 避免了复杂的继承层次
- 更好的代码复用
- 支持零成本抽象

### 3. 内存安全
- 自动内存管理
- 借用检查器防止数据竞争
- 没有空指针或野指针

### 4. 更灵活的设计
- 可以在同一个GridSystem上使用不同的builder
- 支持泛型和trait约束
- 可以轻松添加新的builder实现

## 示例实现

### 简单网格构建器

```rust
struct SimpleGridBuilder {
    width: usize,
    height: usize,
}

impl GridBuilder for SimpleGridBuilder {
    fn build_grid_system(&mut self, grid: &mut GridSystem) -> Result<(), GridError> {
        // 1. 创建所有单元格
        let mut cells = vec![vec![]; self.height];
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

        // 2. 创建连接
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
```

## 与原C++的语义一致性

### 1. 构建时机
- C++：在子类构造函数中调用`buildGridSystem()`
- Rust：通过`build_with()`或`from_builder()`显式调用

### 2. 错误处理
- C++：通常使用异常或返回码
- Rust：使用`Result<(), GridError>`类型安全的错误处理

### 3. 内存管理
- C++：需要手动管理`cells_`和`edgelist_`的内存
- Rust：自动内存管理，无需担心内存泄漏

## 扩展性

这个trait设计支持轻松添加新的网格类型：

```rust
// 六角形网格
struct HexagonalGridBuilder { /* ... */ }

// 3D网格
struct Grid3DBuilder { /* ... */ }

// 不规则网格
struct IrregularGridBuilder { /* ... */ }
```

每个新的builder只需要实现`GridBuilder` trait即可，无需修改现有代码。

## 测试支持

trait设计使得单元测试更加容易：

```rust
#[test]
fn test_custom_grid() {
    let builder = CustomGridBuilder::new(/* params */);
    let grid = GridSystem::from_builder(builder).unwrap();
    
    // 测试网格属性
    assert_eq!(grid.get_cells_count(), expected_count);
    // 测试连接性
    assert!(grid.validate_structure().is_ok());
}
```

## 总结

`GridBuilder` trait 成功地将C++的虚函数多态转换为Rust的trait系统，在保持原有语义的同时，提供了更好的类型安全、内存安全和可组合性。这个设计为后续实现具体的网格类型（如正交2D网格）提供了坚实的基础。 