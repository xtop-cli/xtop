<h1>Usage</h1>

<hr>

<h2 id="table-of-contents">Table of Contents</h2>

<ul>
  <li><a href="#keybindings">Keybindings</a></li>
  <li><a href="#modules">Modules</a></li>
  <li><a href="#help-overlay">Help Overlay</a></li>
  <li><a href="#command-palette">Command Bar</a></li>
  <li><a href="#full-screen-mode">Full-Screen Mode</a></li>
  <li><a href="#responsive-layouts">Responsive Layouts</a></li>
  <li><a href="#command-line-commands">Command-Line Commands</a></li>
</ul>

<hr>

<h2 id="keybindings">Keybindings</h2>

<table>
  <thead>
    <tr>
      <th>Key</th>
      <th>Action</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><kbd>q</kbd></td>
      <td>Quit application (saves configuration)</td>
    </tr>
    <tr>
      <td><kbd>?</kbd></td>
      <td>Toggle help overlay</td>
    </tr>
    <tr>
      <td><kbd>t</kbd></td>
      <td>Next color theme</td>
    </tr>
    <tr>
      <td><kbd>T</kbd></td>
      <td>Previous color theme</td>
    </tr>
    <tr>
      <td><kbd>l</kbd></td>
      <td>Next layout (cycles through the layout modes, the Detail presets, then custom layouts)</td>
    </tr>
    <tr>
      <td><kbd>f</kbd></td>
      <td>Toggle full-screen mode for the current widget</td>
    </tr>
    <tr>
      <td><kbd>F</kbd></td>
      <td>Cycle full-screen focus through all available widgets</td>
    </tr>
    <tr>
      <td><kbd>/</kbd> / <kbd>ctrl+p</kbd></td>
      <td>Open the command bar</td>
    </tr>
    <tr>
      <td><kbd>ctrl+f</kbd></td>
      <td>Filter processes</td>
    </tr>
    <tr>
      <td><kbd>Enter</kbd></td>
      <td>Confirm filter / select command</td>
    </tr>
    <tr>
      <td><kbd>Backspace</kbd></td>
      <td>Delete character / return to the previous page</td>
    </tr>
    <tr>
      <td><kbd>Esc</kbd></td>
      <td>Cancel search / close command bar or help</td>
    </tr>
  </tbody>
</table>

<hr>

<h2 id="modules">Modules</h2>

<ol>
  <li>
    <p><strong>Header</strong> -- Shows system uptime, load average, current theme name, and active layout mode.</p>
  </li>
  <li>
    <p><strong>CPU</strong> -- Horizontal usage bars for each CPU core. If hardware sensors are available, displays the maximum CPU temperature.</p>
  </li>
  <li>
    <p><strong>Memory</strong> -- Gauges for RAM and Swap usage, plus a line chart showing RAM usage over a configurable history window.</p>
  </li>
  <li>
    <p><strong>Storage</strong> -- Disk usage gauges per mount point showing capacity and used space.</p>
  </li>
  <li>
    <p><strong>Network</strong> -- Total downloaded (RX) and uploaded (TX) data per interface, with current transfer rates.</p>
  </li>
  <li>
    <p><strong>Disk I/O</strong> -- Read and write speeds per disk device in bytes per second.</p>
  </li>
  <li>
    <p><strong>Processes</strong> -- Scrolling list of processes sorted by CPU usage (top 200 by default; override with <code>XTOP_MAX_PROCESSES</code>), with live search filtering by process name.</p>
  </li>
  <li>
    <p><strong>GPU</strong> -- GPU usage gauges (available on supported hardware).</p>
  </li>
  <li>
    <p><strong>Battery</strong> -- Battery charge level gauges (available on supported hardware).</p>
  </li>
</ol>

<hr>

<h2 id="help-overlay">Help Overlay</h2>

<p>Press <kbd>?</kbd> at any time to display a full list of available keybindings on screen. Press <kbd>?</kbd> again or <kbd>Esc</kbd> to close the overlay.</p>

<hr>

<h2 id="process-search">Process Filter</h2>

<p>Filtering narrows the process list in real time:</p>

<ul>
  <li>Press <kbd>ctrl+f</kbd> to enter the filter, or open the command bar with <kbd>/</kbd> / <kbd>ctrl+p</kbd> and choose <strong>Processes (filter) →</strong>.</li>
  <li>Type any query -- results filter instantly by process name.</li>
  <li>Press <kbd>Enter</kbd> to confirm the filter, <kbd>Esc</kbd> to cancel, <kbd>Backspace</kbd> to delete characters.</li>
  <li>A centered overlay shows the current <code>Filter: query_</code> input.</li>
</ul>

<hr>

<h2 id="command-palette">Command Bar</h2>

<p>Press <kbd>/</kbd> or <kbd>ctrl+p</kbd> to open the command bar. It is a
searchable, layered list of actions and pages: <strong>Main</strong>
(themes/layouts/plugins/processes, full-screen, help, sort, random theme,
exit), <strong>Themes</strong>, <strong>Layouts</strong> and
<strong>Plugins</strong>. The Processes entry enters the interactive process
filter. The Plugins page toggles each compiled plugin on or off with
<kbd>Enter</kbd>; the choice is persisted in <code>config.json</code>.</p>

<p>Type to filter the current page, use <kbd>up</kbd>/<kbd>down</kbd> to
move, <kbd>Enter</kbd> to run or select, <kbd>Backspace</kbd> on an empty
filter to return to the main page, and <kbd>Esc</kbd> to close.</p>

<hr>

<h2 id="full-screen-mode">Full-Screen Mode</h2>

<ul>
  <li>Press <kbd>f</kbd> to toggle full-screen mode for the currently focused widget. The widget expands to fill the entire terminal area (minus the header bar).</li>
  <li>Press <kbd>F</kbd> to cycle full-screen focus through widgets in sequence: CPU, Memory, Storage, Network, Processes, Disk I/O, GPU, Battery, then exit.</li>
</ul>

<hr>

<h2 id="responsive-layouts">Responsive Layouts</h2>

<p>The interface adapts automatically to the terminal size:</p>

<table>
  <thead>
    <tr>
      <th>Terminal Size</th>
      <th>Behavior</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>Dashboard mode, 100+ cols and 28+ rows</td>
      <td>Full dashboard layout with 2 columns</td>
    </tr>
    <tr>
      <td>Dashboard mode, 80&ndash;99 cols or under 28 rows</td>
      <td>Compact layout</td>
    </tr>
    <tr>
      <td>Dashboard mode, narrower than 80 cols</td>
      <td>Vertically stacked layout</td>
    </tr>
    <tr>
      <td>Other modes</td>
      <td>The requested mode layout (Vertical, Horizontal, CPU/Memory/Network/Process Focus)</td>
    </tr>
    <tr>
      <td>Narrower than 60 cols or shorter than 14 rows</td>
      <td>Minimal layout: CPU, Memory, and Processes only</td>
    </tr>
    <tr>
      <td>Smaller than 40 x 8</td>
      <td>Warning message displayed (terminal too small)</td>
    </tr>
  </tbody>
</table>

<hr>

<h2 id="command-line-commands">Command-Line Commands</h2>

<p>Besides the TUI, <code>xtop</code> ships management subcommands. Help is
layered: <code>xtop --help</code> lists every command,
<code>xtop &lt;group&gt; --help</code> lists the commands in a group, and
<code>xtop &lt;group&gt; &lt;command&gt; --help</code> lists the arguments and
notes for one command. Help is written to stdout; usage errors are written
to stderr and exit with status 2. See <a href="commands.md">commands.md</a>
for the complete reference.</p>

<table>
  <thead>
    <tr><th>Command</th><th>Description</th></tr>
  </thead>
  <tbody>
    <tr><td><code>xtop --help</code></td><td>Show global help</td></tr>
    <tr><td><code>xtop status [section] [--json]</code></td><td>Full snapshot or one section: cpu, memory, swap, load, uptime, disks, disk-io, network, battery, gpu</td></tr>
    <tr><td><code>xtop processes [--json]</code></td><td>One-shot process list</td></tr>
    <tr><td><code>xtop watch [--jsonl]</code></td><td>Periodic system snapshots</td></tr>
    <tr><td><code>xtop info</code> / <code>xtop doctor</code></td><td>Build/path information and local diagnostics</td></tr>
    <tr><td><code>xtop update</code></td><td>Update through xpm, pacman, cargo or the release binary installer</td></tr>
    <tr><td><code>xtop --gen-config</code></td><td>Create <code>config.json</code>, <code>themes/x.jsonc</code> and <code>layouts/dashboard.jsonc</code></td></tr>
    <tr><td><code>xtop --change-theme &lt;theme&gt;</code></td><td>Change the active theme (persists; running instances follow within one tick)</td></tr>
    <tr><td><code>xtop theme ...</code></td><td>Manage installed and downloadable themes; run <code>xtop theme --help</code></td></tr>
    <tr><td><code>xtop layout ...</code></td><td>Manage installed and downloadable layouts; run <code>xtop layout --help</code></td></tr>
    <tr><td><code>xtop addon ...</code></td><td>Manage installable themes and layouts; run <code>xtop addon --help</code></td></tr>
    <tr><td><code>xtop config ...</code></td><td>Manage configuration and asset paths; run <code>xtop config --help</code></td></tr>
    <tr><td><code>xtop mcp</code></td><td>Start the MCP server (stdio transport) for AI agents</td></tr>
    <tr><td><code>xtop plugin ...</code></td><td>Manage compiled plugins; run <code>xtop plugin --help</code></td></tr>
    <tr><td><code>xtop widget ...</code></td><td>Manage compiled widget packs; run <code>xtop widget --help</code></td></tr>
  </tbody>
</table>

<p>Plugins and widget packs are compile-time: the commands register crates
and feature flags in the kernel's sources; enable a feature in the
<code>[features]</code> default list (or build with
<code>--features &lt;name&gt;</code>) and rebuild to include it. Widget-pack
details live in <a href="customization.md#widget-packs">customization.md
(&ldquo;Widget Packs&rdquo;)</a>, plugin details in
<a href="plugin.md">plugin.md</a>. Widgets can also be provided at runtime,
without recompiling the kernel: build with
<code>--features plugin-wasm,plugin-external</code> and drop <code>.wasm</code>
modules or helper-process descriptors into the config dir — see
<a href="customization.md#runtime-widgets">customization.md
(&ldquo;Runtime Widgets&rdquo;)</a>.</p>

<hr>

<p align="center">
  <a href="../README.md">Back to README</a>
</p>
