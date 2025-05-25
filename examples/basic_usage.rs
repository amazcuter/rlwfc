/**
 * @file basic_usage.rs
 * @author amazcuter (amazcuter@outlook.com)
 * @brief RLWFC库基本使用示例
 *        展示如何创建网格系统、添加单元格、创建边连接，以及进行方向查询
 * @version 1.0
 * @date 2025-01-25
 *
 * @copyright Copyright (c) 2025
 */

use RLWFC::{GridSystem, Cell, Direction4, DirectionTrait};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== RLWFC 基本使用示例 ===\n");

    // 1. 创建网格系统
    let mut grid = GridSystem::new();
    println!("1. 创建了新的网格系统");

    // 2. 添加单元格（创建2x2网格）
    println!("\n2. 创建2x2网格:");
    let cells = vec![
        vec![
            grid.add_cell(Cell::with_id(0)),
            grid.add_cell(Cell::with_id(1))
        ],
        vec![
            grid.add_cell(Cell::with_id(2)),
            grid.add_cell(Cell::with_id(3))
        ],
    ];

    // 打印网格布局
    println!("   网格布局:");
    println!("   {:?} {:?}", cells[0][0], cells[0][1]);
    println!("   {:?} {:?}", cells[1][0], cells[1][1]);

    // 3. 按标准顺序创建边连接（东向、南向）
    println!("\n3. 创建边连接:");
    for y in 0..2 {
        for x in 0..2 {
            let current = cells[y][x];
            
            // 东向边
            if x + 1 < 2 {
                let east = cells[y][x + 1];
                grid.create_edge(current, east)?;
                println!("   {:?} -> {:?} (东向)", current, east);
            }
            
            // 南向边
            if y + 1 < 2 {
                let south = cells[y + 1][x];
                grid.create_edge(current, south)?;
                println!("   {:?} -> {:?} (南向)", current, south);
            }
        }
    }

    // 4. 展示网格统计
    println!("\n4. 网格统计:");
    println!("{}", grid.get_statistics());

    // 5. 测试方向查询
    println!("\n5. 方向查询测试:");
    let center_cell = cells[0][0]; // 左上角单元格
    
    println!("   以单元格 {:?} 为中心:", center_cell);
    
    for direction in Direction4::all_directions() {
        if let Some(neighbor) = grid.get_neighbor_by_direction(center_cell, direction) {
            println!("     {}: {:?}", direction.name(), neighbor);
        } else {
            println!("     {}: None", direction.name());
        }
    }

    // 6. 展示邻居关系
    println!("\n6. 邻居关系:");
    for y in 0..2 {
        for x in 0..2 {
            let cell = cells[y][x];
            let neighbors = grid.get_neighbors(cell);
            println!("   {:?}: neighbors = {:?}", cell, neighbors);
        }
    }

    // 7. 验证网格结构
    println!("\n7. 验证网格结构:");
    match grid.validate_structure() {
        Ok(()) => println!("   ✅ 网格结构验证通过"),
        Err(e) => println!("   ❌ 网格结构验证失败: {}", e),
    }

    // 8. 演示错误处理
    println!("\n8. 错误处理演示:");
    match grid.create_edge(center_cell, center_cell) {
        Ok(_) => println!("   意外：自循环边创建成功"),
        Err(e) => println!("   ✅ 正确捕获错误: {}", e),
    }

    // 9. 使用命名单元格
    println!("\n9. 命名单元格演示:");
    let named_cell = grid.add_cell_with_name(
        Cell::with_name("特殊单元格".to_string()), 
        "special".to_string()
    );
    println!("   添加了命名单元格: {:?}", named_cell);
    
    if let Some(found_cell) = grid.get_cell_by_name("special") {
        println!("   通过名称找到单元格: {:?}", found_cell);
    }

    // 10. 调试信息
    println!("\n10. 详细调试信息:");
    grid.debug_print_neighbors(center_cell);

    println!("\n=== 示例完成 ===");
    Ok(())
}

/// 演示创建更复杂的网格
#[allow(dead_code)]
fn create_3x3_grid() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 3x3 网格示例 ===");
    
    let mut grid = GridSystem::with_capacity(9, 12); // 预分配容量
    
    // 创建3x3网格
    let mut cells = Vec::new();
    for y in 0..3 {
        let mut row = Vec::new();
        for x in 0..3 {
            let cell = grid.add_cell_with_name(
                Cell::with_id((y * 3 + x) as u32),
                format!("cell_{}_{}", x, y)
            );
            row.push(cell);
        }
        cells.push(row);
    }
    
    // 创建双向连接（每个方向都有对应的边）
    for y in 0..3 {
        for x in 0..3 {
            let current = cells[y][x];
            
            // 东西连接
            if x + 1 < 3 {
                let east = cells[y][x + 1];
                grid.create_edge(current, east)?; // 东向
                grid.create_edge(east, current)?; // 西向
            }
            
            // 南北连接
            if y + 1 < 3 {
                let south = cells[y + 1][x];
                grid.create_edge(current, south)?; // 南向
                grid.create_edge(south, current)?; // 北向
            }
        }
    }
    
    println!("创建了3x3双向连接网格");
    println!("{}", grid.get_statistics());
    
    // 测试中心单元格的四个方向
    let center = cells[1][1];
    println!("\n中心单元格 {:?} 的四个方向:", center);
    for direction in Direction4::all_directions() {
        if let Some(neighbor) = grid.get_neighbor_by_direction(center, direction) {
            println!("  {}: {:?}", direction.name(), neighbor);
        }
    }
    
    Ok(())
} 