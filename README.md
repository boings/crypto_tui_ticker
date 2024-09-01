# Crypto TUI Ticker

![Rust](https://img.shields.io/badge/Rust-1.65%2B-orange?style=for-the-badge&logo=rust)
![License](https://img.shields.io/badge/License-MIT-blue?style=for-the-badge)
![Build Status](https://img.shields.io/badge/Build-Passing-brightgreen?style=for-the-badge&logo=githubactions)
![Issues](https://img.shields.io/github/issues/boings/crypto_tui_ticker?style=for-the-badge)

Crypto TUI is a terminal-based application that provides live updates of cryptocurrency tickers using a stylish, interactive table. Built with Rust and `ratatui`, this application lets you sort tickers by columns, visually track price changes, and navigate effortlessly using keyboard shortcuts.

## Features

- 📈 **Live Ticker Updates**: Stay up-to-date with the latest crypto prices streamed directly from Binance.
- 🔍 **Sortable Columns**: Easily sort each column to analyze the data in the way that suits you best.
- 🌈 **Color-Coded Price Changes**: Instantly see price changes with green and red highlights.
- 🎨 **Customizable Table Styles**: Switch color palettes on the fly to suit your visual preferences.

## Demo

![Demo](demo.gif)

## Installation

1. **Clone the repository:**

   ```bash
   git clone https://github.com/username/crypto-tui.git
   cd crypto-tui
   ```

2. Install Rust if you haven't already:

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Run the application:**

   ```bash
   cargo run
   ```

## Usage

### keybindings

- Navigation: Use the arrow keys, j/k, or h/l to navigate up, down, left, and right.
- Sorting: Press Tab to cycle through columns or r to reverse the sort order.
- Change Colors: Use → or ← to switch between color palettes.
- Quit: Press Esc or q to exit the application.
- Tab: Cycle through columns to sort by.
- r: Reverse the sort order.

License

Crypto TUI Ticker is licensed under the MIT License. See LICENSE for more details.

Acknowledgements

• Built with ❤️ using Rust and ratatui.

Contact

Created by boings - feel free to contact me!
