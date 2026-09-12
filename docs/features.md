<h1>Features</h1>

<p>Detailed breakdown of xtop's monitoring and interface capabilities.</p>

<hr>

<h2 id="system-monitoring">System Monitoring</h2>

<h3 id="cpu">CPU</h3>

<ul>
  <li>Usage percentage per core and per thread, displayed as horizontal gauges.</li>
  <li>Maximum CPU temperature reading when hardware sensors are available.</li>
  <li>Color-coded bars that visually indicate load levels.</li>
</ul>

<h3 id="memory">Memory</h3>

<ul>
  <li>RAM usage gauge showing used, total, and percentage.</li>
  <li>Swap usage gauge.</li>
  <li>Historical line chart tracking RAM usage over time.</li>
  <li>Configurable number of history data points.</li>
</ul>

<h3 id="network">Network</h3>

<ul>
  <li>Real-time upload (TX) and download (RX) tracking per network interface.</li>
  <li>Total data transferred displayed alongside current transfer speeds.</li>
</ul>

<h3 id="storage">Storage</h3>

<ul>
  <li>Disk usage gauges per mount point showing used, available, and total space.</li>
  <li>Visual percentage bars for each mounted filesystem.</li>
</ul>

<h3 id="disk-io">Disk I/O</h3>

<ul>
  <li>Read and write speed tracking per disk device.</li>
  <li>Displayed in bytes per second with automatic unit scaling.</li>
</ul>

<h3 id="processes">Processes</h3>

<ul>
  <li>Scrolling list of running processes sorted by CPU usage.</li>
  <li>Live search filtering by process name.</li>
  <li>Displays process name, CPU usage, and memory usage.</li>
</ul>

<h3 id="gpu">GPU</h3>

<ul>
  <li>GPU usage gauges: real data on Linux (NVIDIA via <code>nvidia-smi</code>, AMD/Intel via <code>/sys/class/drm</code>); NVIDIA is also detected via <code>nvidia-smi</code> on macOS and Windows. AMD/Intel GPUs and Apple GPUs expose no public utilization API, so the widget stays empty for them (no fabricated readings).</li>
</ul>

<h3 id="battery">Battery</h3>

<ul>
  <li>Battery charge level gauges: real data on Linux (<code>/sys/class/power_supply</code>), macOS (<code>pmset</code>) and Windows (<code>GetSystemPowerStatus</code>, aggregate battery). Hosts without a battery show the honest empty state.</li>
</ul>

<hr>

<h2 id="theming">Theming</h2>

<ul>
  <li>12 color schemes: 12 JSONC theme files ship in <code>assets/themes/</code> and are embedded in the binary as first-run seeding templates.</li>
  <li>Custom themes defined as JSONC files with an explicit background/foreground pair and a 16-entry hex color palette (legacy 16-slot-only files still load).</li>
  <li>Always-legible themes: every theme is contrast-normalized at load (WCAG floors for text and mark roles, UX8.2); shipped files stay canonical.</li>
  <li>Instant theme cycling with <kbd>t</kbd> (next) and <kbd>T</kbd> (previous).</li>
  <li>Palette reference in <a href="colors.md">colors.md</a>.</li>
</ul>

<hr>

<h2 id="layouts">Layouts</h2>

<ul>
  <li>7 built-in layout modes: Dashboard, Vertical, Horizontal, CPU Focus, Memory Focus, Network Focus, Process Focus, plus 3 Detail preset layouts (<code>Detail Dashboard</code>, <code>Detail Network</code>, <code>Detail Processes</code>) that cycle after the modes and showcase per-widget display options.</li>
  <li>Custom layouts defined as JSONC files with a recursive split/widget tree; every widget instance can carry an <code>options</code> object (CPU basis, cores, ifaces, ...) driven by the layout file.</li>
  <li>Full-screen mode for any widget toggled with <kbd>f</kbd>.</li>
  <li>Responsive design that adapts to terminal width and height automatically.</li>
  <li>Minimal fallback layout for very small terminals.</li>
</ul>

<hr>

<h2 id="alert-thresholds">Alert Thresholds</h2>

<ul>
  <li>Visual warnings when CPU, memory, or disk usage exceeds configurable limits.</li>
  <li>Color changes to red and warning indicators in widget titles.</li>
</ul>

<hr>

<h2 id="persistence">Persistence</h2>

<ul>
  <li>Current theme, layout, update interval, history points, alert thresholds and glyph style are saved automatically on quit.</li>
  <li>Configuration is stored as <code>config.json</code> in the platform config dir: <code>~/.config/xtop/</code> on Linux, <code>~/Library/Application Support/xtop/</code> on macOS, <code>%APPDATA%\xtop</code> on Windows.</li>
</ul>

<hr>

<h2 id="extensibility">Extensibility</h2>

<ul>
  <li>Plugins, widget packs, extensions and effects are compile-time (Cargo features + git dependencies); a plain build needs none of them.</li>
  <li>Runtime widgets (opt-in <code>plugin-wasm</code> and <code>plugin-external</code> features) load widgets that are not compiled into the kernel: sandboxed <code>.wasm</code> modules (wasmi, fuel and memory limits) or one helper process per widget in any language over line-delimited JSON. They register through the plugin path, keep precedence over packs, and are referenced in layouts by name — see <a href="customization.md#runtime-widgets">customization.md</a>.</li>
  <li>External themers can switch the active theme with <code>xtop --ct &lt;theme&gt;</code>; running instances follow the persisted change within one tick.</li>
</ul>

<hr>

<p align="center">
  <a href="../README.md">Back to README</a>
</p>
