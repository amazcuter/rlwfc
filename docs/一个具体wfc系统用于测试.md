# 一个基本瓷砖集的实现

main.cpp
```cpp
#include <iostream>
#include "Orthogonal2d.h"
#include <random>
#include <map>

class basicWFCManager : public WFC::Manager::WFCManager<int>
{
public:
    virtual void initialize() override
    {
        // 在此初始化瓷砖集与网格系统
        grid_ = new WFC::GridSystem::Orthogonal2DGrid(20, 20); // 网格大小
        tileSet_ = new WFC::TileSet::SquareTileSet();

        grid_->buildGridSystem();
        tileSet_->buildTileSet();

        // 为每个cell建立单元WFC附加信息对象
        // 创建一个 Mersenne Twister 随机数生成器，并用种子初始化
        std::mt19937 generator(42);

        // 定义一个均匀分布，范围为 [minVal, maxVal]
        std::uniform_int_distribution<int> distribution(0, 99999);
        using CellList = WFC::GridSystem::GridSystem::CellList;
        CellList *celllist = grid_->getAllCells();
        Tiles allPossibility = tileSet_->getAllTiles();
        minEntropyCell = *(*celllist).begin();
        for (auto &cell : *celllist)
        {
            wfcCellData[cell] = CellwfcData(distribution(generator), allPossibility);
            if (wfcCellData[cell].entropy < wfcCellData[minEntropyCell].entropy)
            {
                minEntropyCell = cell;
            }
        }
    }
};

struct Color
{
    int r, g, b;
};

basicWFCManager *wfcm = nullptr;
WFC::GridSystem::Orthogonal2DGrid *grid = nullptr;
int colorToInt(const Color &color) { return (color.r << 16) | (color.g << 8) | color.b; }

extern "C"
{
    using namespace std;
    __declspec(dllexport) vector<vector<int>> getImage()
    {
        using namespace WFC;
        // 构造一个二维Vector,每个元素是一个像素点，大小最好是600*600，元素的值为colorToInt(Color{r, g, b})，其中r,g,b取值为0-255
        vector<vector<int>> image(1200, vector<int>(1200));
        for (CellID cellId : *(grid->getAllCells()))
        {
            pair<int, int> position = grid->getPosition(cellId);
            int y =  position.first;
            int x =  position.second;
            int x_range_begin = x * 60;
            int x_range_end = x_range_begin + 60;
            int y_range_begin = y * 60;
            int y_range_end = y_range_begin + 60;
            switch (wfcm->getCellState(cellId))
            {
            case Manager::WFCManager<int>::State::Noncollapsed:
                // 未坍塌，将图填为灰色
                for (int j = y_range_begin; j < y_range_end; j++)
                {
                    for (int i = x_range_begin; i < x_range_end; i++)
                    {
                        image[i][j] = colorToInt(Color{127, 127, 127});
                    }
                }
                break;

            case Manager::WFCManager<int>::State::conflict:
                // 冲突，将图填为红色
                for (int j = y_range_begin; j < y_range_end; j++)
                {
                    for (int i = x_range_begin; i < x_range_end; i++)
                    {
                        image[i][j] = colorToInt(Color{255, 0, 0});
                    }
                }
                break;

            case Manager::WFCManager<int>::State::Collapsed:
                Tile<int> *data = wfcm->getCollapsedCellData(cellId);
                // 已坍塌,填上瓷砖
                for (int j = y_range_begin; j < y_range_end; j++)
                {
                    for (int i = x_range_begin; i < x_range_end; i++)
                    {
                        if ((i % 60 > 19 && i % 60 < 40 && j % 60 > 19 && j % 60 < 40) ||                    // 中间
                            (data->edge[0] && i % 60 >= 0 && i % 60 <= 19 && j % 60 > 19 && j % 60 < 40) ||  // 上
                            (data->edge[2] && i % 60 > 19 && i % 60 < 40 && j % 60 >= 0 && j % 60 <= 19) ||  // 左
                            (data->edge[3] && i % 60 > 19 && i % 60 < 40 && j % 60 >= 40 && j % 60 <= 59) || // 右
                            (data->edge[1] && i % 60 >= 40 && i % 60 <= 59 && j % 60 > 19 && j % 60 < 40)    // 下
                        )
                        {
                            image[i][j] = colorToInt(Color{0, 0, 0});
                        }
                        else
                        {
                            if (i % 60 == 0 || i % 60 == 59 || j % 60 == 0 || j % 60 == 59)
                            {
                                image[i][j] = colorToInt(Color{0, 255, 0});
                            }
                            else
                            {
                                image[i][j] = colorToInt(Color{255, 255, 255});
                            }
                        }
                    }
                }
                break;
            }
        }
        return image;
    }
    __declspec(dllexport) void run()
    {
        // 运行一次的操作
        grid = (WFC::GridSystem::Orthogonal2DGrid *)wfcm->runStep();
    }
    __declspec(dllexport) void del()
    {
        // 释放内存操作
        if (wfcm != nullptr)
        {
            delete wfcm;
            wfcm = nullptr;
        }
    }
    __declspec(dllexport) void newIter()
    {
        del();
        // 刷新系统状态
        wfcm = new basicWFCManager;
        wfcm->initialize();
        grid = (WFC::GridSystem::Orthogonal2DGrid *)wfcm->getGrid();
    }
    __declspec(dllexport) map<string, int> getData()
    {
        // 获取系统统计数据，暂时不考虑
        return map<string, int>();
    }
    __declspec(dllexport) vector<string> getInfo()
    {
        // 获取文件信息
        return vector<string>{"basicOrthogonal2d-noEPT-noCOR.dll", "基本正交二维WFC系统", "王玉坤", "2025.1.27"};
    }
}
```

orthogonal2d.h
```cpp
#pragma once
/**
 * @file Orthogonal2d.h
 * @author amazcuter (amazcuter@outlook.com)
 * @brief 此文件为简单正交二维WFC系统头文件，网格系统中每个网格有四个边，每个边有两种可能，瓷砖集一共16种
 * @version 0.2
 * @date 2025-01-26
 *
 * @copyright Copyright (c) 2025
 *
 */
#include "WFCManager.h"

namespace WFC
{
    namespace GridSystem
    {
        class Orthogonal2DGrid : public WFC::GridSystem::GridSystem
        {
        private:
            int width, height;
            std::unordered_map<CellID, std::pair<int, int>> position;

        public:
            std::pair<int, int> getPosition(CellID id)
            {
                return position[id];
            }
            virtual void buildGridSystem() override
            {
                // 假设我们有一个宽度为width，高度为height的网格。
                // CreateEdge(cell, new Cell());
                CellID cells[width][height];
                // step 1 创建单元
                for (int y = 0; y < height; ++y)
                {
                    for (int x = 0; x < width; ++x)
                    {
                        CellID newCell = new Cell();
                        cells[x][y] = newCell;
                        cells_.push_back(newCell);
                        position[newCell] = std::pair<int, int>{x, y};
                    }
                }
                // step 2 创建链接
                // !!!注意，这里的CreateEdge方法是创建边的一半，只创建一个方向的链接，这样的目的是保持边的对应性
                // 边界情况使用nullptr
                // !!!这里的边的顺序要记住，在下面judgePossibility时会使用到!!!
                for (int y = 0; y < height; ++y)
                {
                    for (int x = 0; x < width; ++x)
                    {
                        CellID cell = cells[x][y];
                        // 上
                        if (y > 0)
                            CreateEdge(cell, cells[x][y - 1]);
                        else
                            CreateEdge(cell, nullptr);
                        // 下
                        if (y < height - 1)
                            CreateEdge(cell, cells[x][y + 1]);
                        else
                            CreateEdge(cell, nullptr);
                        // 左
                        if (x > 0)
                            CreateEdge(cell, cells[x - 1][y]);
                        else
                            CreateEdge(cell, nullptr);
                        // 右
                        if (x < width - 1)
                            CreateEdge(cell, cells[x + 1][y]);
                        else
                            CreateEdge(cell, nullptr);
                    }
                }
            }

        public:
            Orthogonal2DGrid(int w, int h) : width(w), height(h) {}
        };
    }
    namespace TileSet
    {
        class SquareTileSet : public TileSet<int>
        {
            // 假设EdgeData是一个表示边类型的整数。
        private:
            // 边的种类
            static const int EDGE_TYPES = 2;
            // 每个瓷砖的边数
            static const int EDGES_PER_TILE = 4;

        public:
            void addTile(int up, int down, int left, int right)
            {
                TileID<int> tile = new Tile<int>();
                tile->edge = {up, down, left, right};
                tile->weight = 10;
                tiles_.push_back(tile);
            }

            virtual void buildTileSet() override
            {
                // 建立一个四边组成瓷砖，两种边类型的瓷砖集
                // ALL0 全0
                addTile(0, 0, 0, 0);

                // ALL1 全1
                addTile(1, 1, 1, 1);

                // EPT 端点
                // addTile(1, 0, 0, 0);
                // addTile(0, 1, 0, 0);
                // addTile(0, 0, 1, 0);
                // addTile(0, 0, 0, 1);

                // CAN 通道
                addTile(1, 1, 0, 0);
                addTile(0, 0, 1, 1);

                // COR 拐角
                // addTile(1, 0, 0, 1);
                // addTile(0, 1, 0, 1);
                // addTile(1, 0, 1, 0);
                // addTile(0, 1, 1, 0);

                // TJU 三岔路口
                addTile(0, 1, 1, 1);
                addTile(1, 0, 1, 1);
                addTile(1, 1, 0, 1);
                addTile(1, 1, 1, 0);
            }

            virtual bool judgePossibility(std::vector<Tiles> neighborPossibility, TileID<int> possibility) override
            {
                // 检查此可能性与所有邻居的兼容性
                // !!!注意边的"顺序"!!!
                // // std::cout << "possibility:";
                // for (auto i : possibility->edge)
                // {
                //     std::cout << i << ' ';
                // }
                // std::cout << std::endl;
                // std::cout << "\tneighborPossibility size:" << neighborPossibility.size() << std::endl;
                for (int i = 0; i < neighborPossibility.size(); ++i)
                {
                    // 这里edge[i]所相邻的单元的对应边为edge[EDGES_PER_TILE - i - 1]
                    // std::cout << "\t\tedge:" << i << std::endl;
                    // std::cout << "\t\tneighborPossibility[i].size():" << neighborPossibility[i].size() << std::endl;
                    bool edgeOk = false;
                    for (TileID<int> tile : neighborPossibility[i])
                    {

                        // std::cout << "\t\t\ttile:";
                        // for (auto e : tile->edge)
                        // {
                        //     std::cout << e << ' ';
                        // }
                        int neighborIndex;
                        switch (i)
                        {
                        case 0:
                            neighborIndex = 1;
                            break;
                        case 1:
                            neighborIndex = 0;
                            break;
                        case 2:
                            neighborIndex = 3;
                            break;
                        case 3:
                            neighborIndex = 2;
                            break;
                        }
                        if (tile->edge[neighborIndex] == possibility->edge[i])
                        {
                            // std::cout << "pass" << std::endl;
                            edgeOk = true;
                            break;
                        }
                        else
                        {
                            // std::cout << "bad" << std::endl;
                        }
                    }
                    if (!edgeOk)
                    {
                        return false;
                    }
                }
                return true;
            }
        };
    }
}
```