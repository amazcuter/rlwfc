/**
 * @file tile_system_demo.rs
 * @author amazcuter (amazcuter@outlook.com)
 * @brief 瓷砖系统使用示例
 *        展示TileSet和TileSetVirtual trait的使用，包括不同类型的瓷砖约束实现
 * @version 1.0
 * @date 2025-01-25
 *
 * @copyright Copyright (c) 2025
 */

use rlwfc::{TileSet, TileSetVirtual, Tile, TileId};

// =============================================================================
// 简单瓷砖集实现 - 字符串边数据
// =============================================================================

/// 简单瓷砖集，使用字符串作为边数据
/// 
/// 这是最基本的瓷砖集实现，适合演示WFC的基本概念。
/// 每个瓷砖有四个边，用字符串表示边的类型。
struct SimpleTileSet {
    tiles: TileSet<&'static str>,
}

impl SimpleTileSet {
    fn new() -> Self {
        Self {
            tiles: TileSet::new(),
        }
    }
}

impl TileSetVirtual<&'static str> for SimpleTileSet {
    fn build_tile_set(&mut self) {
        // 清空现有瓷砖
        self.tiles.clear();
        
        println!("   构建简单瓷砖集...");
        
        // 添加基础瓷砖：草地
        self.tiles.add_tile(vec!["grass", "grass", "grass", "grass"], 50);
        println!("     添加草地瓷砖 (权重: 50)");
        
        // 添加基础瓷砖：水面
        self.tiles.add_tile(vec!["water", "water", "water", "water"], 30);
        println!("     添加水面瓷砖 (权重: 30)");
        
        // 添加过渡瓷砖：草地到水面
        self.tiles.add_tile(vec!["grass", "water", "water", "grass"], 20);
        println!("     添加过渡瓷砖1 (权重: 20)");
        
        self.tiles.add_tile(vec!["water", "grass", "grass", "water"], 20);
        println!("     添加过渡瓷砖2 (权重: 20)");
        
        // 添加角落瓷砖
        self.tiles.add_tile(vec!["grass", "grass", "water", "water"], 15);
        println!("     添加角落瓷砖 (权重: 15)");
        
        println!("   瓷砖集构建完成，总共 {} 个瓷砖", self.tiles.get_tile_count());
    }

    fn judge_possibility(
        &self,
        neighbor_possibilities: &[Vec<TileId>],
        candidate: TileId
    ) -> bool {
        // 获取候选瓷砖
        let Some(candidate_tile) = self.tiles.get_tile(candidate) else {
            return false;
        };
        
        // 检查每个方向的约束
        for (direction, neighbor_ids) in neighbor_possibilities.iter().enumerate() {
            if neighbor_ids.is_empty() {
                continue; // 该方向无约束
            }
            
            // 检查是否与任一邻居瓷砖兼容
            let is_compatible = neighbor_ids.iter().any(|&neighbor_id| {
                if let Some(neighbor_tile) = self.tiles.get_tile(neighbor_id) {
                    // 简单的边匹配：当前瓷砖的该方向边 == 邻居瓷砖的相对方向边
                    let opposite_direction = (direction + 2) % 4; // 相对方向
                    if let (Some(current_edge), Some(neighbor_edge)) = (
                        candidate_tile.get_edge(direction),
                        neighbor_tile.get_edge(opposite_direction)
                    ) {
                        current_edge == neighbor_edge
                    } else {
                        false
                    }
                } else {
                    false
                }
            });
            
            if !is_compatible {
                return false;
            }
        }
        
        true
    }
}

// 为SimpleTileSet提供便捷的访问方法
impl SimpleTileSet {
    fn get_tile_count(&self) -> usize {
        self.tiles.get_tile_count()
    }
    
    fn get_tile(&self, tile_id: TileId) -> Option<&Tile<&'static str>> {
        self.tiles.get_tile(tile_id)
    }
    
    fn get_all_tiles(&self) -> &[Tile<&'static str>] {
        self.tiles.get_all_tiles()
    }
}

// =============================================================================
// 数字瓷砖集实现 - 数字边数据
// =============================================================================

/// 数字瓷砖集，使用整数作为边数据
/// 
/// 展示如何使用数字来表示边的类型，适合更复杂的约束计算。
struct NumericTileSet {
    tiles: TileSet<i32>,
}

impl NumericTileSet {
    fn new() -> Self {
        Self {
            tiles: TileSet::new(),
        }
    }
}

impl TileSetVirtual<i32> for NumericTileSet {
    fn build_tile_set(&mut self) {
        self.tiles.clear();
        
        println!("   构建数字瓷砖集...");
        
        // 使用数字表示不同的边类型
        // 0 = 平原, 1 = 山脉, 2 = 河流
        
        // 平原瓷砖
        self.tiles.add_tile(vec![0, 0, 0, 0], 40);
        println!("     添加平原瓷砖 [0,0,0,0] (权重: 40)");
        
        // 山脉瓷砖
        self.tiles.add_tile(vec![1, 1, 1, 1], 25);
        println!("     添加山脉瓷砖 [1,1,1,1] (权重: 25)");
        
        // 河流瓷砖（直线）
        self.tiles.add_tile(vec![2, 0, 2, 0], 20); // 垂直河流
        self.tiles.add_tile(vec![0, 2, 0, 2], 20); // 水平河流
        println!("     添加河流瓷砖 (权重: 20 各)");
        
        // 过渡瓷砖
        self.tiles.add_tile(vec![0, 1, 1, 0], 15); // 平原到山脉
        self.tiles.add_tile(vec![1, 0, 0, 1], 15); // 山脉到平原
        println!("     添加过渡瓷砖 (权重: 15 各)");
        
        println!("   数字瓷砖集构建完成，总共 {} 个瓷砖", self.tiles.get_tile_count());
    }

    fn judge_possibility(
        &self,
        neighbor_possibilities: &[Vec<TileId>],
        candidate: TileId
    ) -> bool {
        let Some(candidate_tile) = self.tiles.get_tile(candidate) else {
            return false;
        };
        
        for (direction, neighbor_ids) in neighbor_possibilities.iter().enumerate() {
            if neighbor_ids.is_empty() {
                continue;
            }
            
            let is_compatible = neighbor_ids.iter().any(|&neighbor_id| {
                if let Some(neighbor_tile) = self.tiles.get_tile(neighbor_id) {
                    let opposite_direction = (direction + 2) % 4;
                    if let (Some(&current_edge), Some(&neighbor_edge)) = (
                        candidate_tile.get_edge(direction),
                        neighbor_tile.get_edge(opposite_direction)
                    ) {
                        // 数字边需要完全匹配
                        current_edge == neighbor_edge
                    } else {
                        false
                    }
                } else {
                    false
                }
            });
            
            if !is_compatible {
                return false;
            }
        }
        
        true
    }
}

impl NumericTileSet {
    fn get_tile_count(&self) -> usize {
        self.tiles.get_tile_count()
    }
    
    fn get_tile(&self, tile_id: TileId) -> Option<&Tile<i32>> {
        self.tiles.get_tile(tile_id)
    }
}

// =============================================================================
// 主函数 - 演示所有瓷砖集
// =============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 瓷砖系统综合示例 ===\n");

    // 1. 基础TileSet演示
    println!("1. 基础TileSet使用:");
    demonstrate_basic_tileset()?;

    // 2. 简单瓷砖集演示
    println!("\n2. 简单瓷砖集 (字符串边):");
    demonstrate_simple_tileset()?;

    // 3. 数字瓷砖集演示
    println!("\n3. 数字瓷砖集 (整数边):");
    demonstrate_numeric_tileset()?;

    // 4. 约束判断演示
    println!("\n4. 约束判断测试:");
    demonstrate_constraint_checking()?;

    println!("\n=== 示例完成 ===");
    Ok(())
}

/// 演示基础TileSet的使用
fn demonstrate_basic_tileset() -> Result<(), Box<dyn std::error::Error>> {
    let mut tile_set = TileSet::new();
    
    println!("   创建基础瓷砖集...");
    
    // 添加一些基础瓷砖
    let tile1 = tile_set.add_tile(vec!["A", "B", "C", "D"], 10);
    let tile2 = tile_set.add_tile(vec!["B", "A", "D", "C"], 15);
    let tile3 = tile_set.add_tile(vec!["C", "D", "A", "B"], 5);
    
    println!("   添加了 {} 个瓷砖", tile_set.get_tile_count());
    
    // 展示瓷砖信息
    for (i, tile) in tile_set.get_all_tiles().iter().enumerate() {
        println!("   瓷砖 {}: ID={}, 权重={}, 边={:?}", 
                 i, tile.id, tile.weight, tile.edges);
    }
    
    // 测试瓷砖兼容性
    if let (Some(t1), Some(t2)) = (tile_set.get_tile(tile1), tile_set.get_tile(tile2)) {
        let compatible = t1.is_compatible_with(t2, 0); // 方向0
        println!("   瓷砖0和瓷砖1在方向0兼容: {}", compatible);
    }
    
    Ok(())
}

/// 演示简单瓷砖集
fn demonstrate_simple_tileset() -> Result<(), Box<dyn std::error::Error>> {
    let mut simple_tileset = SimpleTileSet::new();
    
    // 构建瓷砖集
    simple_tileset.build_tile_set();
    
    // 展示所有瓷砖
    println!("\n   瓷砖详情:");
    for (i, tile) in simple_tileset.get_all_tiles().iter().enumerate() {
        println!("     瓷砖 {}: 边={:?}, 权重={}", 
                 i, tile.edges, tile.weight);
    }
    
    // 测试约束判断
    println!("\n   约束判断测试:");
    test_constraint_judgment(&simple_tileset);
    
    Ok(())
}

/// 演示数字瓷砖集
fn demonstrate_numeric_tileset() -> Result<(), Box<dyn std::error::Error>> {
    let mut numeric_tileset = NumericTileSet::new();
    
    // 构建瓷砖集
    numeric_tileset.build_tile_set();
    
    // 展示瓷砖统计
    println!("\n   瓷砖统计:");
    let mut edge_types = std::collections::HashMap::new();
    for tile in numeric_tileset.tiles.get_all_tiles() {
        for &edge in &tile.edges {
            *edge_types.entry(edge).or_insert(0) += 1;
        }
    }
    
    for (edge_type, count) in edge_types {
        let type_name = match edge_type {
            0 => "平原",
            1 => "山脉",
            2 => "河流",
            _ => "未知",
        };
        println!("     边类型 {} ({}): {} 个边", edge_type, type_name, count);
    }
    
    Ok(())
}

/// 演示约束判断
fn demonstrate_constraint_checking() -> Result<(), Box<dyn std::error::Error>> {
    let mut tileset = SimpleTileSet::new();
    tileset.build_tile_set();
    
    println!("   测试不同约束情况:");
    
    // 情况1: 无约束
    let no_constraints: Vec<Vec<TileId>> = vec![];
    let result1 = tileset.judge_possibility(&no_constraints, 0);
    println!("     无约束情况: {}", if result1 { "可放置" } else { "不可放置" });
    
    // 情况2: 单方向约束
    let single_constraint = vec![vec![0], vec![], vec![], vec![]]; // 只有东向有约束
    let result2 = tileset.judge_possibility(&single_constraint, 1);
    println!("     单方向约束: {}", if result2 { "可放置" } else { "不可放置" });
    
    // 情况3: 多方向约束
    let multi_constraints = vec![vec![0], vec![1], vec![], vec![]]; // 东向和南向都有约束
    let result3 = tileset.judge_possibility(&multi_constraints, 2);
    println!("     多方向约束: {}", if result3 { "可放置" } else { "不可放置" });
    
    // 情况4: 不存在的瓷砖
    let result4 = tileset.judge_possibility(&single_constraint, 999);
    println!("     不存在瓷砖: {}", if result4 { "可放置" } else { "不可放置" });
    
    Ok(())
}

/// 测试约束判断逻辑
fn test_constraint_judgment(tileset: &SimpleTileSet) {
    // 创建测试场景：两个相邻瓷砖
    let grass_tile_id = 0; // 草地瓷砖
    let water_tile_id = 1; // 水面瓷砖
    
    // 测试相同类型瓷砖的兼容性
    let same_type_constraint = vec![vec![grass_tile_id], vec![], vec![], vec![]];
    let compatible_with_same = tileset.judge_possibility(&same_type_constraint, grass_tile_id);
    println!("     草地瓷砖与草地瓷砖兼容: {}", compatible_with_same);
    
    // 测试不同类型瓷砖的兼容性  
    let diff_type_constraint = vec![vec![water_tile_id], vec![], vec![], vec![]];
    let compatible_with_diff = tileset.judge_possibility(&diff_type_constraint, grass_tile_id);
    println!("     草地瓷砖与水面瓷砖兼容: {}", compatible_with_diff);
    
    // 测试过渡瓷砖的兼容性
    if let Some(transition_tile) = tileset.get_tile(2) {
        println!("     过渡瓷砖边: {:?}", transition_tile.edges);
        let transition_compatible = tileset.judge_possibility(&same_type_constraint, 2);
        println!("     过渡瓷砖与草地瓷砖兼容: {}", transition_compatible);
    }
} 