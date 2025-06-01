/**
 * @file grid_builder_demo.rs
 * @author amazcuter (amazcuter@outlook.com)
 * @brief GridBuilder trait 使用示例
 *        展示如何实现不同类型的网格构建器，包括线性网格、2D网格和环形网格
 * @version 1.0
 * @date 2025-01-25
 *
 * @copyright Copyright (c) 2025
 */

use rlwfc::{GridSystem, GridBuilder, Cell, GridError};

// =============================================================================
// 线性网格构建器 - 简单的链式连接
// =============================================================================

/// 线性网格构建器
/// 
/// 创建一条线性链接的单元格序列，每个单元格连接到下一个单元格。
/// 这是最简单的网格类型，适合演示GridBuilder的基本用法。
struct LinearGridBuilder {
    length: usize,
}

impl LinearGridBuilder {
    fn new(length: usize) -> Self {
        Self { length }
    }
}

impl GridBuilder for LinearGridBuilder {
    fn build_grid_system(&mut self, grid: &mut GridSystem) -> Result<(), GridError> {
        if self.length == 0 {
            return Ok(());
        }

        // 创建所有单元格
        let mut cells = Vec::with_capacity(self.length);
        for i in 0..self.length {
            let cell_id = grid.add_cell_with_name(
                Cell::with_id(i as u32),
                format!("linear_cell_{}", i)
            );
            cells.push(cell_id);
        }

        // 创建链式连接
        for i in 0..self.length - 1 {
            grid.create_edge(cells[i], Some(cells[i + 1]))?;
        }

        Ok(())
    }

    fn get_dimensions(&self) -> Vec<usize> {
        vec![self.length]
    }

    fn get_grid_type_name(&self) -> &'static str {
        "LinearGrid"
    }
}

// =============================================================================
// 2D正交网格构建器 - 标准的矩形网格
// =============================================================================

/// 2D正交网格构建器
/// 
/// 创建标准的矩形网格，每个内部单元格有四个邻居（东、南、西、北）。
/// 这是最常用的网格类型，适合大多数2D WFC应用。
struct Orthogonal2DBuilder {
    width: usize,
    height: usize,
}

impl Orthogonal2DBuilder {
    fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }
}

impl GridBuilder for Orthogonal2DBuilder {
    fn build_grid_system(&mut self, grid: &mut GridSystem) -> Result<(), GridError> {
        if self.width == 0 || self.height == 0 {
            return Ok(());
        }

        // Step 1: 创建所有单元格
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

        // Step 2: 按标准顺序创建连接（东向然后南向）
        for y in 0..self.height {
            for x in 0..self.width {
                let current = cells[y][x];
                
                // 连接到右边（东向）
                if x + 1 < self.width {
                    grid.create_edge(current, Some(cells[y][x + 1]))?;
                }
                
                // 连接到下面（南向）
                if y + 1 < self.height {
                    grid.create_edge(current, Some(cells[y + 1][x]))?;
                }
            }
        }

        Ok(())
    }

    fn get_dimensions(&self) -> Vec<usize> {
        vec![self.width, self.height]
    }

    fn get_grid_type_name(&self) -> &'static str {
        "Orthogonal2D"
    }
}

// =============================================================================
// 环形网格构建器 - 循环连接的网格
// =============================================================================

/// 环形网格构建器
/// 
/// 创建一个环形连接的单元格序列，最后一个单元格连接回第一个单元格。
/// 这种拓扑结构在某些特殊的WFC应用中很有用。
struct RingGridBuilder {
    size: usize,
}

impl RingGridBuilder {
    fn new(size: usize) -> Self {
        Self { size }
    }
}

impl GridBuilder for RingGridBuilder {
    fn build_grid_system(&mut self, grid: &mut GridSystem) -> Result<(), GridError> {
        if self.size < 3 {
            return Err(GridError::InvalidDirection); // 环形至少需要3个节点
        }

        // 创建所有单元格
        let mut cells = Vec::with_capacity(self.size);
        for i in 0..self.size {
            let cell_id = grid.add_cell_with_name(
                Cell::with_id(i as u32),
                format!("ring_cell_{}", i)
            );
            cells.push(cell_id);
        }

        // 创建环形连接
        for i in 0..self.size {
            let next = (i + 1) % self.size;
            grid.create_edge(cells[i], Some(cells[next]))?;
        }

        Ok(())
    }

    fn get_dimensions(&self) -> Vec<usize> {
        vec![self.size] // 环形网格只有一个维度
    }

    fn get_grid_type_name(&self) -> &'static str {
        "RingGrid"
    }
}

// =============================================================================
// 主函数 - 演示所有构建器
// =============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== GridBuilder Trait 综合示例 ===\n");

    // 1. 线性网格演示
    println!("1. 线性网格构建器:");
    demonstrate_linear_grid()?;

    // 2. 2D正交网格演示
    println!("\n2. 2D正交网格构建器:");
    demonstrate_2d_grid()?;

    // 3. 环形网格演示
    println!("\n3. 环形网格构建器:");
    demonstrate_ring_grid()?;

    // 4. 构建器对比
    println!("\n4. 构建器对比:");
    compare_grid_builders()?;

    println!("\n=== 示例完成 ===");
    Ok(())
}

/// 演示线性网格构建器
fn demonstrate_linear_grid() -> Result<(), Box<dyn std::error::Error>> {
    let linear_builder = LinearGridBuilder::new(5);
    let linear_grid = GridSystem::from_builder(linear_builder)?;
    
    println!("   类型: LinearGrid");
    println!("   单元格: {}, 边: {}", 
             linear_grid.get_cells_count(), 
             linear_grid.get_edges_count());
    
    // 验证线性连接
    if let Some(first_cell) = linear_grid.get_cell_by_name("linear_cell_0") {
        let neighbors = linear_grid.get_neighbors(first_cell);
        println!("   第一个单元格的邻居数: {}", neighbors.len());
    }

    validate_and_report(&linear_grid, "线性网格");
    Ok(())
}

/// 演示2D正交网格构建器
fn demonstrate_2d_grid() -> Result<(), Box<dyn std::error::Error>> {
    let grid_builder = Orthogonal2DBuilder::new(3, 3);
    let grid_2d = GridSystem::from_builder(grid_builder)?;
    
    println!("   类型: Orthogonal2D");
    println!("   单元格: {}, 边: {}", 
             grid_2d.get_cells_count(), 
             grid_2d.get_edges_count());
    
    // 测试中心单元格的连接
    if let Some(center_cell) = grid_2d.get_cell_by_name("cell_1_1") {
        let neighbors = grid_2d.get_neighbors(center_cell);
        println!("   中心单元格的邻居数: {}", neighbors.len());
        
        // 演示方向查询
        use rlwfc::{Direction4, DirectionTrait};
        println!("   方向查询:");
        for direction in Direction4::all_directions() {
            if let Some(neighbor) = grid_2d.get_neighbor_by_direction(center_cell, direction) {
                println!("     {}: {:?}", direction.name(), neighbor);
            } else {
                println!("     {}: None", direction.name());
            }
        }
    }

    validate_and_report(&grid_2d, "2D网格");
    Ok(())
}

/// 演示环形网格构建器
fn demonstrate_ring_grid() -> Result<(), Box<dyn std::error::Error>> {
    let ring_builder = RingGridBuilder::new(6);
    let ring_grid = GridSystem::from_builder(ring_builder)?;
    
    println!("   类型: RingGrid");
    println!("   单元格: {}, 边: {}", 
             ring_grid.get_cells_count(), 
             ring_grid.get_edges_count());
    
    // 验证环形连接：每个节点都应该有1个出边
    let mut all_have_one_neighbor = true;
    for cell_id in ring_grid.get_all_cells() {
        let neighbors = ring_grid.get_neighbors(cell_id);
        if neighbors.len() != 1 {
            all_have_one_neighbor = false;
            break;
        }
    }
    println!("   所有节点都有1个邻居: {}", all_have_one_neighbor);

    validate_and_report(&ring_grid, "环形网格");
    Ok(())
}

/// 比较不同构建器的特性
fn compare_grid_builders() -> Result<(), Box<dyn std::error::Error>> {
    println!("   | 类型     | 单元格 | 边数 | 验证结果 |");
    println!("   |----------|--------|------|----------|");

    // 线性网格
    {
        let builder = LinearGridBuilder::new(5);
        let grid = GridSystem::from_builder(builder)?;
        let validation = if grid.validate_structure().is_ok() { "✅" } else { "❌" };
        println!("   | {:8} | {:6} | {:4} | {:8} |", 
                 "线性(5)",
                 grid.get_cells_count(),
                 grid.get_edges_count(),
                 validation);
    }

    // 2D网格
    {
        let builder = Orthogonal2DBuilder::new(3, 3);
        let grid = GridSystem::from_builder(builder)?;
        let validation = if grid.validate_structure().is_ok() { "✅" } else { "❌" };
        println!("   | {:8} | {:6} | {:4} | {:8} |", 
                 "2D(3x3)",
                 grid.get_cells_count(),
                 grid.get_edges_count(),
                 validation);
    }

    // 环形网格
    {
        let builder = RingGridBuilder::new(6);
        let grid = GridSystem::from_builder(builder)?;
        let validation = if grid.validate_structure().is_ok() { "✅" } else { "❌" };
        println!("   | {:8} | {:6} | {:4} | {:8} |", 
                 "环形(6)",
                 grid.get_cells_count(),
                 grid.get_edges_count(),
                 validation);
    }

    Ok(())
}

/// 验证网格并报告结果
fn validate_and_report(grid: &GridSystem, grid_type: &str) {
    match grid.validate_structure() {
        Ok(()) => println!("   ✅ {}结构验证通过", grid_type),
        Err(e) => println!("   ❌ {}结构验证失败: {:?}", grid_type, e),
    }
} 