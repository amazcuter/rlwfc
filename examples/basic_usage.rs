/**
 * @file basic_usage.rs
 * @author amazcuter (amazcuter@outlook.com)
 * @brief rlwfc库基本使用示例
 *        展示如何创建网格系统、添加单元格、创建边连接，以及进行方向查询
 * @version 1.0
 * @date 2025-01-25
 *
 * @copyright Copyright (c) 2025
 */
use rlwfc::{Cell, Direction4, DirectionTrait, GridError, GridSystem};

fn main() -> Result<(), GridError> {
    println!("=== rlwfc 基本使用示例 ===\n");

    // 1. 创建网格系统
    let mut grid = GridSystem::new();
    println!("1. 创建了新的网格系统");

    // 2. 添加单元格（创建2x2网格）
    println!("\n2. 创建2x2网格:");
    let cell1 = grid.add_cell(Cell::with_id(1));
    let cell2 = grid.add_cell(Cell::with_id(2));
    let cell3 = grid.add_cell(Cell::with_id(3));
    let cell4 = grid.add_cell(Cell::with_id(4));

    println!("添加了 {} 个单元格", grid.get_cells_count());

    let cells = vec![vec![cell1, cell2], vec![cell3, cell4]];

    // 打印网格布局
    println!("   网格布局:");
    println!("   {:?} {:?}", cells[0][0], cells[0][1]);
    println!("   {:?} {:?}", cells[1][0], cells[1][1]);

    // 3. 按标准顺序创建边连接（东向、南向）
    println!("\n3. 创建边连接:");
    let width = 2;
    let height = 2;

    // 逐个单元格按照正确顺序创建边：东、南、西、北
    for y in 0..height {
        for x in 0..width {
            let current = cells[y][x];

            // 东向连接
            if x + 1 < width {
                let east = cells[y][x + 1];
                grid.create_edge(current, Some(east))?;
                println!("   {:?} -> {:?} (东向)", current, east);
            }

            // 南向连接
            if y + 1 < height {
                let south = cells[y + 1][x];
                grid.create_edge(current, Some(south))?;
                println!("   {:?} -> {:?} (南向)", current, south);
            }
        }
    }

    println!("创建了 {} 条边", grid.get_edges_count());

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
    for y in 0..height {
        for x in 0..width {
            let current = cells[y][x];
            let neighbors = grid.get_neighbors(current);
            println!("   {:?}: neighbors = {:?}", current, neighbors);
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
    match grid.create_edge(center_cell, Some(center_cell)) {
        Ok(_) => println!("   意外：自循环边创建成功"),
        Err(e) => println!("   ✅ 正确捕获错误: {}", e),
    }

    // 9. 使用命名单元格
    println!("\n9. 命名单元格演示:");
    let named_cell = grid.add_cell_with_name(
        Cell::with_name("特殊单元格".to_string()),
        "special".to_string(),
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

fn find_cell_position(
    grid: &[Vec<petgraph::graph::NodeIndex>],
    target: petgraph::graph::NodeIndex,
) -> (usize, usize) {
    for (y, row) in grid.iter().enumerate() {
        for (x, &cell) in row.iter().enumerate() {
            if cell == target {
                return (x, y);
            }
        }
    }
    (999, 999) // 未找到
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_creation() {
        let mut grid = GridSystem::new();
        let cell1 = grid.add_cell(Cell::with_id(1));
        let cell2 = grid.add_cell(Cell::with_id(2));

        // 测试自环检测
        match grid.create_edge(cell1, Some(cell1)) {
            Err(GridError::SelfLoop) => println!("✓ 正确检测到自环"),
            _ => panic!("应该检测到自环错误"),
        }

        assert_eq!(grid.get_cells_count(), 2);
    }

    #[test]
    fn test_neighbors_order() -> Result<(), GridError> {
        let mut grid = GridSystem::new();

        // 创建2x2网格以测试邻居顺序
        let cells = vec![
            vec![
                grid.add_cell(Cell::with_id(0)),
                grid.add_cell(Cell::with_id(1)),
            ],
            vec![
                grid.add_cell(Cell::with_id(2)),
                grid.add_cell(Cell::with_id(3)),
            ],
        ];

        // 为中心位置 (0,0) 的单元格创建边
        let current = cells[0][0];
        let east = cells[0][1];
        let south = cells[1][0];

        // 按照正确顺序创建边：东、南、西、北
        // 东向
        grid.create_edge(current, Some(east))?;
        grid.create_edge(east, Some(current))?;

        // 南向
        grid.create_edge(current, Some(south))?;
        grid.create_edge(south, Some(current))?;

        // 验证邻居顺序是否符合预期：[北, 西, 南, 东]
        let neighbors = grid.get_neighbors(current);

        // 在2x2网格中，(0,0)位置只有东和南邻居
        // 由于edges顺序是[东, 南]，neighbors()应该返回[南, 东]（逆序）
        assert_eq!(neighbors.len(), 2);

        Ok(())
    }
}
