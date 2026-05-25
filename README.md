# 养基场

实时基金估值、持仓收益和桌面悬浮小组件。

养基场最初是一个纯前端基金估值看板，现在同时支持 Web 页面和 Windows 桌面端。它可以添加基金自选、查看实时估值和重仓走势，也可以在桌面上常驻一个透明小面板，用于快速查看总估值、日收益、持有收益和持仓标签。

在线预览：[https://fund-baby.ningzhengsheng.cn/](https://fund-baby.ningzhengsheng.cn/)

![首页截图](app/assets/fund-baby-img1.png)
![详情截图](app/assets/fund-baby-img2.png)

## 功能

- 实时估值：根据基金代码获取单位净值、估算净值、涨跌幅和更新时间。
- 重仓跟踪：展示基金重仓股票，并结合行情数据估算盘中表现。
- 自选管理：支持添加、删除、分组、搜索和本地持久化。
- 持仓收益：可录入持仓份额和成本，计算当前估值、日收益、持有收益和收益率。
- 桌面小组件：Windows 下支持无边框透明悬浮窗、置顶、拖动、托盘菜单和快捷键。
- F2 快捷键：按 `F2` 在主面板和日收益小条之间切换；恢复主面板时会自动刷新一次。
- 托盘设置：支持开机自启动、透明度调节、显示/隐藏和退出。
- 自动刷新：默认每 10 秒刷新一次，也支持手动刷新。
- 纯前端数据：主要通过公开行情接口获取数据，不需要自建后端服务。

## 桌面端

桌面端基于 Tauri 2 构建，适合把基金看板作为日常悬浮窗口使用。

常用操作：

- `F2`：主面板和日收益小条切换。
- 日收益小条：默认出现在工作区左下角，可拖动调整位置，并在本次运行期间记住上次位置。
- 托盘图标左键：显示主面板并刷新。
- 托盘菜单：开机自启动、透明度、显示、隐藏到托盘、退出。
- 面板内刷新按钮：立即拉取最新行情。
- 编辑模式：维护基金列表、持仓份额和成本。

日收益小条只显示当前持仓总日收益和更新时间，默认避开任务栏显示在左下角。它不是 Windows 任务栏组件，而是一个普通的置顶透明窗口。

## 技术栈

- Next.js 16
- React 18
- Tauri 2
- ECharts
- dayjs
- 原生 CSS

数据来源包括天天基金、东方财富、腾讯财经等公开接口。接口可用性和数据准确性受第三方服务影响。

## 本地开发

环境要求：

- Node.js `>=20.9.0`
- npm
- Rust stable
- Windows 桌面端构建需要 Tauri 相关系统依赖

安装依赖：

```bash
npm install
```

启动 Web 开发服务：

```bash
npm run dev
```

访问：

```text
http://localhost:3000
```

启动桌面开发模式：

```bash
npm run dev:desktop
```

## 构建

构建 Web 静态产物：

```bash
npm run build
```

构建 Windows 桌面应用：

```bash
npm run build:desktop
```

构建完成后，桌面可执行文件位于：

```text
src-tauri/target/release/fund-baby.exe
```

安装包位于：

```text
src-tauri/target/release/bundle/nsis/
```

如果构建时报 `fund-baby.exe` 无法删除，通常是旧程序还在运行，先退出托盘程序或结束进程后再构建。

## 环境变量

反馈功能使用 Web3Forms。不开启反馈功能时可以不配置。

复制示例文件：

```bash
cp env.example .env.local
```

可选变量：

```text
NEXT_PUBLIC_WEB3FORMS_ACCESS_KEY=你的 Web3Forms Access Key
```

## Docker

构建镜像：

```bash
docker build -t fund-baby .
```

启动容器：

```bash
docker run -d -p 3000:3000 --name fund-baby fund-baby
```

使用 docker compose：

```bash
docker compose up -d
```

## 部署

项目可以部署到 GitHub Pages、Vercel 或其他静态站点托管服务。

当前仓库包含 GitHub Actions 配置。启用 GitHub Pages 时，可在仓库设置中选择：

```text
Settings -> Pages -> Build and deployment -> Source: GitHub Actions
```

推送到 `main` 分支后会触发自动构建和部署。

## 免责声明

本项目仅供个人学习、记录和参考。所有行情、估值和收益计算结果都可能存在延迟、误差或接口不可用情况，不构成任何投资建议。投资决策请以基金公司、交易平台和官方披露信息为准。

## License

本项目基于 [GNU Affero General Public License v3.0](LICENSE) 开源。

如果你修改本项目并通过网络服务向用户提供使用，也需要向用户提供对应源代码，并继续使用相同协议。

## 联系

- GitHub：[https://github.com/zhengshengning](https://github.com/zhengshengning)
- 博客：[https://ningzhengsheng.cn](https://ningzhengsheng.cn)
