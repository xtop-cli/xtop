<div align="center">
<img src="https://raw.githubusercontent.com/xtop-cli/web/main/public/img/logo.png" width="200px" alt="Xtop logo" />
</div>
<div align="center">

![Rust](https://img.shields.io/badge/Rust-1.87%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)
![Platform](https://img.shields.io/badge/platform-linux%20%7C%20macos%20%7C%20windows-lightgrey)
![ratatui](https://img.shields.io/badge/built%20with-ratatui-red)

*A cross-platform TUI system monitor written in Rust. Uses ratatui for the terminal interface and sysinfo for real-time system metrics.*

<p align="center">
  <a href="https://xtop-cli.github.io/web/img/previews/preview1.webp">
    <img src="https://xtop-cli.github.io/web/img/previews/preview1.webp" alt="Main preview" width="900"/>
  </a>
</p>

</div>

<hr>

<h2>Contents</h2>

<ul>
  <li><a href="#features">Features</a></li>
  <li><a href="#previews">Previews</a></li>
  <li><a href="#quick-install">Quick Install</a></li>
  <li><a href="#quick-start">Quick Start</a></li>
  <li><a href="#documentation">Documentation</a></li>
  <li><a href="#contributing">Contributing</a></li>
  <li><a href="#license">License</a></li>
  <li><a href="#x">X</a></li>
</ul>

<hr>

<h2 id="features">Features</h2>

<ul>
  <li>CPU usage per core with temperature sensing</li>
  <li>RAM and Swap monitoring with historical chart</li>
  <li>Network RX/TX tracking per interface</li>
  <li>Storage and Disk I/O visualization</li>
  <li>Process list with live search, CPU/Memory/PID/Name sorting and ▲/▼ direction toggling</li>
  <li>Battery monitoring (real probes on Linux, macOS and Windows)</li>
  <li>GPU monitoring (NVIDIA via <code>nvidia-smi</code> on any platform; AMD/Intel via Linux sysfs probes)</li>
  <li>12 color themes with custom theme support via JSONC</li>
  <li>7 built-in layout modes + 3 Detail preset layouts, with custom layout support via JSONC</li>
  <li>Full-screen mode for any widget</li>
  <li>Configurable alert thresholds</li>
  <li>Persistent configuration</li>
</ul>

<p>See <a href="docs/features.md">docs/features.md</a> for a detailed feature breakdown.</p>

<hr>

<h2 id="previews">Previews</h2>

  <p align="center">
    <a href="https://xtop-cli.github.io/web/img/previews/preview2.webp">
      <img src="https://xtop-cli.github.io/web/img/previews/preview2.webp" alt="xtop preview 2" width="100%"/>
    </a>
  </p>

<details>
  <summary><strong>More...</strong></summary>


  <p align="center">
    <a href="https://xtop-cli.github.io/web/img/previews/preview3.webp">
      <img src="https://xtop-cli.github.io/web/img/previews/preview3.webp" alt="xtop preview 3" width="100%"/>
    </a>
  </p>

  <p align="center">
    <a href="https://xtop-cli.github.io/web/img/previews/preview4.webp">
      <img src="https://xtop-cli.github.io/web/img/previews/preview4.webp" alt="xtop preview 4" width="100%"/>
    </a>
  </p>

  <p align="center">
    <a href="https://xtop-cli.github.io/web/img/previews/preview5.webp">
      <img src="https://xtop-cli.github.io/web/img/previews/preview5.webp" alt="xtop preview 5" width="100%"/>
    </a>
  </p>

  <p align="center">
    <a href="https://xtop-cli.github.io/web/img/previews/preview6.webp">
      <img src="https://xtop-cli.github.io/web/img/previews/preview6.webp" alt="xtop preview 6" width="100%"/>
    </a>
  </p>

  <p align="center">
    <a href="https://xtop-cli.github.io/web/img/previews/preview7.webp">
      <img src="https://xtop-cli.github.io/web/img/previews/preview7.webp" alt="xtop preview 7" width="100%"/>
    </a>
  </p>

  <p align="center">
    <a href="https://xtop-cli.github.io/web/img/previews/preview8.webp">
      <img src="https://xtop-cli.github.io/web/img/previews/preview8.webp" alt="xtop preview 8" width="100%"/>
    </a>
  </p>
</details>

<hr>

<h2 id="quick-install">Quick Install</h2>

<h3>Linux</h3>

<pre><code>curl -fsSL https://raw.githubusercontent.com/xtop-cli/xtop/main/install.sh | bash</code></pre>

<h3>Windows (PowerShell)</h3>

<pre><code>irm https://raw.githubusercontent.com/xtop-cli/xtop/main/install.ps1 | iex</code></pre>

<h3>macOS and other platforms</h3>

<p><code>install.sh</code> targets Linux package managers. On macOS (or any other
platform) install with cargo:</p>

<pre><code>cargo install --git https://github.com/xtop-cli/xtop --all-features</code></pre>

<p>or build from source (see below).</p>

<h3>Build from Source</h3>

<pre><code>git clone https://github.com/xtop-cli/xtop.git
cd xtop
cargo run --release</code></pre>

<p>For detailed installation instructions, see <a href="docs/installation.md">docs/installation.md</a>.</p>

<hr>

<h2 id="quick-start">Quick Start</h2>

<p>Run <code>xtop</code> after installation. Key controls:</p>

<table>
  <thead>
    <tr><th>Key</th><th>Action</th></tr>
  </thead>
  <tbody>
    <tr><td><kbd>q</kbd></td><td>Quit (saves config)</td></tr>
    <tr><td><kbd>?</kbd></td><td>Toggle help overlay</td></tr>
    <tr><td><kbd>t</kbd> / <kbd>T</kbd></td><td>Next / previous theme</td></tr>
    <tr><td><kbd>l</kbd></td><td>Next layout mode</td></tr>
    <tr><td><kbd>f</kbd> / <kbd>F</kbd></td><td>Toggle / cycle full-screen</td></tr>
    <tr><td><kbd>/</kbd></td><td>Search processes</td></tr>
  </tbody>
</table>

<p>For full usage details, see <a href="docs/usage.md">docs/usage.md</a>.</p>

<hr>

<h2 id="documentation">Documentation</h2>

<ul>
  <li><a href="docs/features.md">Features</a> -- detailed feature breakdown</li>
  <li><a href="docs/installation.md">Installation</a> -- full install and uninstall guide</li>
  <li><a href="docs/usage.md">Usage</a> -- keybindings, modules, help overlay</li>
  <li><a href="docs/configuration.md">Configuration</a> -- config file and settings reference</li>
  <li><a href="docs/customization.md">Customization</a> -- custom themes and layouts</li>
  <li><a href="docs/colors.md">Colors</a> -- palette reference for the 12 shipped themes</li>
  <li><a href="docs/plugin.md">Plugins</a> -- plugin architecture and authoring</li>
  <li><a href="docs/multi-repo.md">Multi-repo architecture</a> -- ecosystem RFC and layout</li>
  <li><a href="ROADMAP.md">Roadmap</a></li>
  <li><a href="CHANGELOG.md">Changelog</a></li>
  <li><a id="contributing" href="CONTRIBUTING.md">Contributing</a></li>
  <li><a id="license" href="LICENSE">License</a></li>
</ul>

<hr>

<div id="x" align="center">
<h2>X</h2>

<a href="https://www.xscriptor.io">Dev</a>
 & 
<a href="https://github.com/xscriptor">Github Profile</a>
 & 
<a href="https://www.xscriptor.com">Xscriptor</a>