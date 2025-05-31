//! # 正交2D WFC系统示例
//! 
//! 这个示例实现了一个具体的WFC系统，对应C++版本的basicWFCManager。
//! 
//! ## 系统特性
//! 
//! - **网格**：10x10的正交2D网格
//! - **瓷砖**：每个瓷砖有4条边，每条边有2种类型（0或1）
//! - **可视化**：生成ASCII艺术来显示WFC状态
//! 
//! ## 瓷砖类型
//! 
//! - ALL0: 全0瓷砖 [0,0,0,0]
//! - ALL1: 全1瓷砖 [1,1,1,1] 
//! - 通道瓷砖: [1,1,0,0], [0,0,1,1]
//! - 三岔路口: [0,1,1,1], [1,0,1,1], [1,1,0,1], [1,1,1,0]

use rlwfc::{
    GridSystem, GridBuilder, TileSet, TileSetVirtual, WfcManager, DefaultInitializer,
    Cell, Tile, GridError, WfcError, TileId, CellState
};

// =============================================================================
// 正交2D网格构建器
// =============================================================================

/// 正交2D网格构建器，对应C++的Orthogonal2DGrid
struct Orthogonal2DGridBuilder {
    width: usize,
    height: usize,
}

impl Orthogonal2DGridBuilder {
    fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }
}

impl GridBuilder for Orthogonal2DGridBuilder {
    fn build_grid_system(&mut self, grid: &mut GridSystem) -> Result<(), GridError> {
        println!("构建 {}x{} 正交2D网格...", self.width, self.height);
        
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
        
        // Step 2: 按照WFC库的要求创建边：东、南、西、北顺序
        // 这样neighbors()会返回：[北, 西, 南, 东] (petgraph逆序)
        for y in 0..self.height {
            for x in 0..self.width {
                let current = cells[y][x];
                
                // 1. 东边 (向右)
                if x < self.width - 1 {
                    grid.create_edge(current, Some(cells[y][x + 1]))?;
                } else {
                    grid.create_edge(current, None)?;
                }
                
                // 2. 南边 (向下)
                if y < self.height - 1 {
                    grid.create_edge(current, Some(cells[y + 1][x]))?;
                } else {
                    grid.create_edge(current, None)?;
                }
                
                // 3. 西边 (向左)
                if x > 0 {
                    grid.create_edge(current, Some(cells[y][x - 1]))?;
                } else {
                    grid.create_edge(current, None)?;
                }
                
                // 4. 北边 (向上)
                if y > 0 {
                    grid.create_edge(current, Some(cells[y - 1][x]))?;
                } else {
                    grid.create_edge(current, None)?;
                }
            }
        }
        
        println!("网格构建完成：{} 个单元格，{} 条边", 
                grid.get_cells_count(), grid.get_edges_count());
        Ok(())
    }
    
    fn get_dimensions(&self) -> Vec<usize> {
        vec![self.width, self.height]
    }
    
    fn get_grid_type_name(&self) -> &'static str {
        "Orthogonal2DGrid"
    }
}

// =============================================================================
// 方形瓷砖集，对应C++的SquareTileSet
// =============================================================================

/// 方形瓷砖集，对应C++的SquareTileSet
struct SquareTileSet {
    tiles: TileSet<i32>,
}

impl SquareTileSet {
    fn new() -> Self {
        Self {
            tiles: TileSet::new(),
        }
    }
    
    /// 添加瓷砖的辅助方法，对应C++的addTile
    fn add_tile(&mut self, north: i32, west: i32, south: i32, east: i32) -> TileId {
        // 按照neighbors()返回顺序：[北, 西, 南, 东]
        self.tiles.add_tile(vec![north, west, south, east], 10)
    }
}

impl TileSetVirtual<i32> for SquareTileSet {
    fn build_tile_set(&mut self) {
        println!("构建方形瓷砖集...");
        self.tiles.clear();
        
        // ALL0 全0瓷砖 [北, 西, 南, 东] = [0, 0, 0, 0]
        self.add_tile(0, 0, 0, 0);
        println!("  添加 ALL0 瓷砖: [0,0,0,0]");
        
        // ALL1 全1瓷砖 [北, 西, 南, 东] = [1, 1, 1, 1]
        self.add_tile(1, 1, 1, 1);
        println!("  添加 ALL1 瓷砖: [1,1,1,1]");
        
        // 垂直通道：北南连通，东西断开 [北, 西, 南, 东] = [1, 0, 1, 0]
        self.add_tile(1, 0, 1, 0);
        
        // 水平通道：东西连通，北南断开 [北, 西, 南, 东] = [0, 1, 0, 1]
        self.add_tile(0, 1, 0, 1);
        println!("  添加通道瓷砖: 垂直[1,0,1,0], 水平[0,1,0,1]");
        
        // 三岔路口瓷砖 (三个方向连通)
        self.add_tile(0, 1, 1, 1); // 西南东三通 [0,1,1,1]
        self.add_tile(1, 0, 1, 1); // 北南东三通 [1,0,1,1]
        self.add_tile(1, 1, 0, 1); // 北西东三通 [1,1,0,1]
        self.add_tile(1, 1, 1, 0); // 北西南三通 [1,1,1,0]
        println!("  添加三岔路口瓷砖: 4种类型");
        
        println!("瓷砖集构建完成，总共 {} 个瓷砖", self.tiles.get_tile_count());
    }
    
    fn judge_possibility(
        &self,
        neighbor_possibilities: &[Vec<TileId>],
        candidate: TileId
    ) -> bool {
        let candidate_tile = match self.tiles.get_tile(candidate) {
            Some(tile) => tile,
            None => return false,
        };
        
        // 检查候选瓷砖与所有邻居的兼容性
        // neighbor_possibilities的索引对应：[北, 西, 南, 东]
        for (direction_index, neighbor_tiles) in neighbor_possibilities.iter().enumerate() {
            if neighbor_tiles.is_empty() {
                continue;
            }
            
            let mut edge_compatible = false;
            let candidate_edge = candidate_tile.edges[direction_index];
            
            for &neighbor_tile_id in neighbor_tiles {
                if let Some(neighbor_tile) = self.tiles.get_tile(neighbor_tile_id) {
                    // 计算邻居瓷砖对应方向的边索引
                    // 边数据顺序：[北, 西, 南, 东]
                    let neighbor_edge_index = match direction_index {
                        0 => 2, // 北边 -> 邻居的南边
                        1 => 3, // 西边 -> 邻居的东边
                        2 => 0, // 南边 -> 邻居的北边
                        3 => 1, // 东边 -> 邻居的西边
                        _ => continue,
                    };
                    
                    let neighbor_edge = neighbor_tile.edges[neighbor_edge_index];
                    
                    // 相邻边必须相等才兼容
                    if candidate_edge == neighbor_edge {
                        edge_compatible = true;
                        break;
                    }
                }
            }
            
            if !edge_compatible {
                return false;
            }
        }
        
        true
    }
    
    fn get_tile(&self, tile_id: TileId) -> Option<&Tile<i32>> {
        self.tiles.get_tile(tile_id)
    }
    
    fn get_tile_count(&self) -> usize {
        self.tiles.get_tile_count()
    }
    
    fn get_all_tile_ids(&self) -> Vec<TileId> {
        self.tiles.get_all_tile_ids()
    }
}

// =============================================================================
// 主要演示逻辑
// =============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("正交2D WFC系统演示");
    println!("对应C++版本的basicWFCManager实现");
    println!();
    
    // 创建小一点的网格以便于观察
    let width = 10;
    let height = 10;
    
    // 1. 创建网格系统
    let grid_builder = Orthogonal2DGridBuilder::new(width, height);
    let grid = GridSystem::from_builder(grid_builder)?;
    
    // 2. 创建瓷砖集
    let tile_set = Box::new(SquareTileSet::new());
    
    // 3. 创建WFC管理器
    let mut wfc_manager = WfcManager::new(grid, tile_set)?;
    
    // 4. 初始化系统
    let mut initializer = DefaultInitializer;
    wfc_manager.initialize_with(&mut initializer)?;
    
    println!("WFC系统初始化完成\n");
    
    // 5. 显示初始状态
    println!("初始状态:");
    print_statistics(&wfc_manager);
    print_ascii_grid(&wfc_manager, width, height);
    
    // 6. 运行WFC算法
    println!("开始WFC算法执行...\n");
    
    let mut step_count = 0;
    let max_steps = 500000;
    
    loop {
        step_count += 1;
        
        match wfc_manager.run_step() {
            Ok(rlwfc::StepResult::Collapsed) => {
                if step_count <= 5 {
                    println!("步骤 {}: 成功坍塌一个单元", step_count);
                }
            }
            Ok(rlwfc::StepResult::ConflictsResolved) => {
                println!("步骤 {}: 解决了冲突", step_count);
            }
            Ok(rlwfc::StepResult::Complete) => {
                println!("步骤 {}: WFC算法完成!", step_count);
                break;
            }
            Ok(rlwfc::StepResult::ConflictResolutionFailed) => {
                println!("步骤 {}: 冲突解决失败", step_count);
                break;
            }
            Err(e) => {
                println!("步骤 {}: 错误 - {:?}", step_count, e);
                break;
            }
            
        }
        
        
        // 每20步显示一次状态
        if step_count % 20 == 0 {
            print_ascii_grid(&wfc_manager, width, height);
        }
        
        if step_count >= max_steps {
            println!("达到最大步数限制 ({})", max_steps);
            break;
        }
    }
    
    // 7. 显示最终结果
    println!("\n最终结果:");
    print_statistics(&wfc_manager);
    print_ascii_grid(&wfc_manager, width, height);
    
    // 8. 演示瓷砖信息
    demonstrate_tiles(&wfc_manager)?;
    
    Ok(())
}

/// 打印系统统计信息
fn print_statistics(manager: &WfcManager<i32>) {
    let grid = manager.get_grid();
    let total_cells = grid.get_cells_count();
    
    let mut uncollapsed_count = 0;
    let mut collapsed_count = 0;
    let mut conflict_count = 0;
    
    for cell_id in grid.get_all_cells() {
        match manager.get_cell_state(cell_id) {
            Ok(rlwfc::CellState::Uncollapsed) => uncollapsed_count += 1,
            Ok(rlwfc::CellState::Collapsed) => collapsed_count += 1,
            Ok(rlwfc::CellState::Conflict) => conflict_count += 1,
            Err(_) => {}
        }
    }
    
    println!("系统统计:");
    println!("  总单元格数: {}", total_cells);
    println!("  已坍塌: {}", collapsed_count);
    println!("  未坍塌: {}", uncollapsed_count);
    println!("  冲突: {}", conflict_count);
    println!("  完成率: {:.1}%", (collapsed_count as f64 / total_cells as f64) * 100.0);
}

/// 打印ASCII网格
fn print_ascii_grid(manager: &WfcManager<i32>, width: usize, height: usize) {
    println!("\nWFC 系统状态可视化:");
    
    // 打印顶部边框
    print!("┏");
    for _ in 0..width * 2 {
        print!("━");
    }
    println!("┓");
    
    for y in 0..height {
        print!("┃");
        for x in 0..width {
            let cell_name = format!("cell_{}_{}", x, y);
            let cell_id = manager.get_grid().get_cell_by_name(&cell_name).unwrap();
            
            let symbol = match manager.get_cell_state(cell_id) {
                Ok(CellState::Collapsed) => {
                    let tile_id = manager.get_collapsed_cell_tile(cell_id).unwrap();
                    let tile = manager.get_tile(tile_id).unwrap();
                    tile_to_symbol(tile)
                }
                Ok(CellState::Uncollapsed) => "?",
                Ok(CellState::Conflict) => "X",
                Err(_) => "E",
            };
            
            print!("{} ", symbol);
        }
        println!("┃");
    }
    
    // 打印底部边框
    print!("┗");
    for _ in 0..width * 2 {
        print!("━");
    }
    println!("┛");
    
    println!("图例: ? = 未坍塌, X = 冲突,   = 空地, ┼ = 四通");
    println!("      ─ │ = 直通道, ┌┐└┘ = 拐角, ├┤┬┴ = 三通, ╵╴╷╶ = 端点");
}

/// 将瓷砖转换为显示符号
fn tile_to_symbol(tile: &Tile<i32>) -> &'static str {
    // 边的顺序是 [北, 西, 南, 东]
    match tile.edges.as_slice() {
        [0, 0, 0, 0] => " ",  // 全0 - 空地
        [1, 1, 1, 1] => "┼",  // 全1 - 四通
        
        // 直通道
        [1, 0, 1, 0] => "│",  // 北南通道 - 垂直
        [0, 1, 0, 1] => "─",  // 西东通道 - 水平
        
        // 拐角 (两个相邻方向的连接)
        [1, 0, 0, 1] => "└",  // 北东拐角
        [0, 0, 1, 1] => "┌",  // 南东拐角  
        [0, 1, 1, 0] => "┐",  // 西南拐角
        [1, 1, 0, 0] => "┘",  // 北西拐角
        
        // 三通 (三个方向的连接)
        [0, 1, 1, 1] => "┬",  // 西南东三通 (右侧T)
        [1, 0, 1, 1] => "├",  // 北南东三通 (左侧T)
        [1, 1, 0, 1] => "┴",  // 北西东三通 (顶部T)
        [1, 1, 1, 0] => "┤",  // 北西南三通 (底部T)
        
        // 其他未定义的组合
        _ => "?",
    }
}

/// 演示瓷砖信息
fn demonstrate_tiles(manager: &WfcManager<i32>) -> Result<(), WfcError> {
    println!("\n瓷砖信息:");
    println!("{}", "─".repeat(40));
    
    let tile_ids = manager.get_all_tile_ids();
    
    for (i, &tile_id) in tile_ids.iter().enumerate() {
        if let Some(tile) = manager.get_tile(tile_id) {
            let symbol = tile_to_symbol(tile);
            println!("瓷砖 {}: 边配置 {:?}, 符号: '{}'", i, tile.edges, symbol);
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_orthogonal_2d_grid_builder() {
        let mut builder = Orthogonal2DGridBuilder::new(3, 3);
        let mut grid = GridSystem::new();
        
        builder.build_grid_system(&mut grid).unwrap();
        
        assert_eq!(grid.get_cells_count(), 9);
        
        // 测试命名查找
        let cell_1_1 = grid.get_cell_by_name("cell_1_1").unwrap();
        assert!(grid.contains_cell(cell_1_1));
    }
    
    #[test]
    fn test_square_tile_set() {
        let mut tile_set = SquareTileSet::new();
        tile_set.build_tile_set();
        
        assert_eq!(tile_set.get_tile_count(), 8); // 2 + 2 + 4 = 8个瓷砖
        
        // 测试ALL0瓷砖
        let all0_tile = tile_set.get_tile(0).unwrap();
        assert_eq!(all0_tile.edges, vec![0, 0, 0, 0]);
        
        // 测试ALL1瓷砖
        let all1_tile = tile_set.get_tile(1).unwrap();
        assert_eq!(all1_tile.edges, vec![1, 1, 1, 1]);
    }
    
    #[test]
    fn test_tile_compatibility() {
        let mut tile_set = SquareTileSet::new();
        tile_set.build_tile_set();
        
        // 测试ALL0瓷砖与自身的兼容性
        let neighbor_constraints = vec![
            vec![0], // 上邻居是ALL0
            vec![],  // 其他方向无约束
            vec![],
            vec![],
        ];
        
        assert!(tile_set.judge_possibility(&neighbor_constraints, 0)); // ALL0应该兼容
    }
} 