/**
 * @file compatibility_test.rs
 * @author amazcuter (amazcuter@outlook.com)
 * @brief 兼容性测试，验证Rust实现与C++WFCManager的使用模式兼容性
 * @version 1.0
 * @date 2025-01-25
 *
 * @copyright Copyright (c) 2025
 */
use rlwfc::{Cell, Direction4, GridSystem, Tile};
use std::collections::HashMap;

/// 模拟C++WFCManager中的CellwfcData结构
#[derive(Debug, Clone)]
struct CellWfcData {
    state: State,
    entropy: f64,
    rand_num: i32,
    possibility: Vec<usize>, // 对应C++中的Tiles
}

/// 模拟C++WFCManager中的State枚举
#[derive(Debug, Clone, PartialEq)]
enum State {
    Collapsed,
    Noncollapsed,
    Conflict,
}

/// 测试核心API兼容性
#[test]
fn test_grid_system_api_compatibility() {
    // 1. 测试C++中 grid_->getAllCells() 的使用模式
    let mut grid = GridSystem::new();

    // 添加一些单元格
    let cells: Vec<_> = (0..4).map(|i| grid.add_cell(Cell::with_id(i))).collect();

    // C++模式：for (const CellID& cell : *grid_->getAllCells())
    let all_cells: Vec<_> = grid.get_all_cells().collect();
    assert_eq!(all_cells.len(), 4);

    // 2. 测试C++中 grid_->getNeighbor(cell) 的使用模式
    grid.create_edge(cells[0], Some(cells[1])).unwrap();
    grid.create_edge(cells[0], Some(cells[2])).unwrap();

    // C++模式：for (CellID neighbor : grid_->getNeighbor(currentCell))
    let neighbors = grid.get_neighbors(cells[0]);
    assert_eq!(neighbors.len(), 2);
    assert!(neighbors.contains(&cells[1]));
    assert!(neighbors.contains(&cells[2]));

    // 3. 测试C++中 grid_->getCellsNum() 的使用模式
    // C++: completedCellCount == grid_->getCellsNum()
    let completed_cell_count = 2;
    let is_complete = completed_cell_count == grid.get_cells_count();
    assert!(!is_complete); // 我们有4个单元格，只完成了2个

    // 4. 测试findEdge兼容性
    assert!(grid.find_edge(cells[0], cells[1]).is_some());
    assert!(grid.find_edge(cells[1], cells[0]).is_none()); // 单向边
}

/// 测试瓷砖系统兼容性
#[test]
fn test_tile_system_compatibility() {
    // 模拟C++中的瓷砖使用
    let tile1 = Tile::new(0, 10, vec!["A", "B", "C", "D"]);
    let tile2 = Tile::new(1, 15, vec!["A", "B", "X", "Y"]);

    // C++模式：tile->weight
    assert_eq!(tile1.weight, 10);
    assert_eq!(tile2.weight, 15);

    // C++模式：tile->edge
    assert_eq!(tile1.edges.len(), 4);
    assert_eq!(tile1.edges[0], "A");

    // C++模式：瓷砖比较 *tile1 == *tile2
    assert_ne!(tile1, tile2);

    let tile3 = Tile::new(0, 10, vec!["A", "B", "C", "D"]);
    assert_eq!(tile1, tile3); // 相同的瓷砖应该相等
}

/// 测试方向系统（新增功能，超越C++版本）
#[test]
fn test_direction_system_enhancement() {
    let mut grid = GridSystem::new();

    // 创建2x2网格，模拟C++中的网格构建
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

    // 按C++中的标准顺序创建边：东向、南向
    let center = cells[0][0];
    let east = cells[0][1];
    let south = cells[1][0];

    grid.create_edge(center, Some(east)).unwrap(); // 东向边
    grid.create_edge(center, Some(south)).unwrap(); // 南向边

    // 新增的方向查询功能（超越C++版本）
    assert_eq!(
        grid.get_neighbor_by_direction(center, Direction4::East),
        Some(east)
    );
    assert_eq!(
        grid.get_neighbor_by_direction(center, Direction4::South),
        Some(south)
    );
    assert_eq!(
        grid.get_neighbor_by_direction(center, Direction4::West),
        None
    );
    assert_eq!(
        grid.get_neighbor_by_direction(center, Direction4::North),
        None
    );
}

/// 模拟C++WFCManager中的数据结构使用
#[test]
fn test_wfc_manager_data_structures() {
    let mut grid = GridSystem::new();
    let cells: Vec<_> = (0..4).map(|i| grid.add_cell(Cell::with_id(i))).collect();

    // 模拟C++中的wfcCellData映射
    let mut wfc_cell_data: HashMap<_, CellWfcData> = HashMap::new();

    // 初始化单元格数据，模拟C++中的CellwfcData构造
    for &cell in &cells {
        wfc_cell_data.insert(
            cell,
            CellWfcData {
                state: State::Noncollapsed,
                entropy: 2.5,
                rand_num: 42,
                possibility: vec![0, 1, 2], // 瓷砖ID列表
            },
        );
    }

    // 模拟C++中的状态检查
    // if (wfcCellData[neighbor].state != State::Noncollapsed)
    for &cell in &cells {
        let data = &wfc_cell_data[&cell];
        assert_eq!(data.state, State::Noncollapsed);
    }

    // 模拟坍塌操作
    if let Some(data) = wfc_cell_data.get_mut(&cells[0]) {
        data.state = State::Collapsed;
        data.entropy = 0.0;
        data.possibility = vec![1]; // 选择了瓷砖1
    }

    // 验证状态更新
    assert_eq!(wfc_cell_data[&cells[0]].state, State::Collapsed);
    assert_eq!(wfc_cell_data[&cells[0]].possibility.len(), 1);
}


/// 模拟C++WFCManager中的传播效果逻辑
#[test]
fn test_propagation_logic_compatibility() {
    let mut grid = GridSystem::new();

    // 创建一个小网格
    let center = grid.add_cell(Cell::with_id(0));
    let east = grid.add_cell(Cell::with_id(1));
    let south = grid.add_cell(Cell::with_id(2));

    grid.create_edge(center, Some(east)).unwrap();
    grid.create_edge(center, Some(south)).unwrap();

    // 模拟C++中的传播逻辑
    // for (CellID neighbor : grid_->getNeighbor(currentCell))
    let neighbors = grid.get_neighbors(center);
    assert_eq!(neighbors.len(), 2);

    // 在C++中会检查 neighbor == nullptr，我们的实现不会有这个问题
    for neighbor in neighbors {
        // 在Rust中，neighbor总是有效的CellId
        assert!(grid.contains_cell(neighbor));

        // 模拟邻居数据访问
        let _neighbor_neighbors = grid.get_neighbors(neighbor);
        // 每个邻居都可以安全访问，不需要空指针检查
    }
}

/// 测试兼容性检查函数模式
#[test]
fn test_compatibility_check_pattern() {
    // 模拟C++中的瓷砖兼容性检查逻辑
    let tile1 = Tile::new(0, 10, vec![1, 2, 3, 4]);
    let tile2 = Tile::new(1, 15, vec![1, 2, 5, 6]);

    // 模拟方向兼容性检查（对应C++中的direction索引）
    // C++中：this->edges[direction] == other.edges[direction]
    let direction = 0; // 东方向
    let is_compatible = tile1.is_compatible_with(&tile2, direction);
    assert!(is_compatible); // 在方向0上都是1，所以兼容

    let direction = 2; // 西方向
    let is_compatible = tile1.is_compatible_with(&tile2, direction);
    assert!(!is_compatible); // 在方向2上分别是3和5，不兼容
}

/// 综合兼容性测试
#[test]
fn test_overall_compatibility() {
    // 这个测试模拟完整的WFCManager工作流程
    let mut grid = GridSystem::new();

    // 1. 构建网格（对应C++中的网格初始化）
    let cells: Vec<_> = (0..9).map(|i| grid.add_cell(Cell::with_id(i))).collect();

    // 2. 创建3x3网格的连接
    for i in 0..3 {
        for j in 0..3 {
            let current_idx = i * 3 + j;
            let current = cells[current_idx];

            // 东向连接
            if j < 2 {
                let east = cells[current_idx + 1];
                grid.create_edge(current, Some(east)).unwrap();
            }

            // 南向连接
            if i < 2 {
                let south = cells[current_idx + 3];
                grid.create_edge(current, Some(south)).unwrap();
            }
        }
    }

    // 3. 验证网格结构
    assert_eq!(grid.get_cells_count(), 9);
    assert!(grid.get_edges_count() > 0);

    // 4. 测试核心WFC操作
    let center = cells[4]; // 中心单元格
    let neighbors = grid.get_neighbors(center);

    // 中心单元格应该有东向和南向邻居
    assert!(neighbors.len() >= 1);

    // 5. 验证所有API都能正常工作
    assert!(grid.validate_structure().is_ok());

    println!("✅ 所有兼容性测试通过！");
}
