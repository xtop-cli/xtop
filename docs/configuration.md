<h1>Configuration</h1>

<p>xtop persists its configuration automatically on quit. The file is
<code>config.json</code> in the platform config directory, next to the
<code>themes/</code> and <code>layouts/</code> folders:</p>

<pre><code>~/.config/xtop/config.json                (Linux)
~/Library/Application Support/xtop/       (macOS)
%APPDATA%\xtop\                           (Windows)
</code></pre>

<p>On Linux you can override the base directory with
<code>$XDG_CONFIG_HOME</code>.</p>

<p>The base <code>x</code> theme and <code>Dashboard</code> layout are
written here on first run (or with <code>xtop --gen-config</code>). Every
other theme or layout is downloaded from the official catalog with
<code>xtop theme download</code>, <code>xtop layout download</code> or
<code>xtop config download</code>. Downloads are recorded in
<code>assets.json</code> with version, author, license and checksum
metadata.</p>

<hr>

<h2 id="keys">Keys</h2>

<table>
  <thead>
    <tr>
      <th>Key</th>
      <th>Type</th>
      <th>Default</th>
      <th>Description</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><code>theme</code></td>
      <td>string</td>
      <td><code>"x"</code></td>
      <td>Currently selected color theme name (one of the 12 shipped or a user theme).</td>
    </tr>
    <tr>
      <td><code>layout_mode</code></td>
      <td>string</td>
      <td><code>"Dashboard"</code></td>
      <td>Built-in layout mode. One of <code>Dashboard</code>, <code>Vertical</code>, <code>Horizontal</code>, <code>CpuFocus</code>, <code>MemoryFocus</code>, <code>NetworkFocus</code>, <code>ProcessFocus</code>. Ignored while <code>layout_name</code> names a valid custom layout.</td>
    </tr>
    <tr>
      <td><code>layout_name</code></td>
      <td>string</td>
      <td><code>""</code></td>
      <td>Name of the active layout when it is a custom (non-built-in) layout. When non-empty and found, it takes precedence over <code>layout_mode</code>.</td>
    </tr>
    <tr>
      <td><code>update_interval_ms</code></td>
      <td>integer</td>
      <td><code>1000</code></td>
      <td>Sampling interval in milliseconds. Clamped to 100&ndash;3,600,000 on load.</td>
    </tr>
    <tr>
      <td><code>history_points</code></td>
      <td>integer</td>
      <td><code>100</code></td>
      <td>Data points retained for the historical charts.</td>
    </tr>
    <tr>
      <td><code>alerts</code></td>
      <td>object</td>
      <td>see below</td>
      <td>Alert thresholds; see <a href="#alert-thresholds">Alert Thresholds</a>.</td>
    </tr>
    <tr>
      <td><code>keybindings</code></td>
      <td>object</td>
      <td>see below</td>
      <td>Key bindings per action; see <a href="#keybindings">Keybindings</a>.</td>
    </tr>
    <tr>
      <td><code>style</code></td>
      <td>object</td>
      <td>see below</td>
      <td>Widget glyph style (chart charset, borders, packs); see <a href="#style">Style</a>.</td>
    </tr>
    <tr>
      <td><code>effect</code></td>
      <td>string (optional)</td>
      <td>absent</td>
      <td>Frame effect applied to every rendered frame: <code>"fade"</code> activates the built-in fade-in (only in builds compiled with the <code>effects</code> feature). Any other value disables effects.</td>
    </tr>
    <tr>
      <td><code>disabled_plugins</code></td>
      <td>array</td>
      <td><code>[]</code></td>
      <td>Plugin ids disabled from the command bar; restored on startup.</td>
    </tr>
  </tbody>
</table>

<h3 id="example">Example</h3>

<pre><code>{
  "theme": "miami",
  "layout_mode": "Dashboard",
  "layout_name": "",
  "update_interval_ms": 1000,
  "history_points": 100,
  "alerts": {
    "cpu_high": 90.0,
    "mem_high": 90.0,
    "disk_high": 90.0
  },
  "keybindings": {
    "quit": ["q"],
    "help": ["?"],
    "next_theme": ["t"],
    "prev_theme": ["T"],
    "next_layout": ["l"],
    "toggle_fullscreen": ["f"],
    "cycle_fullscreen": ["F"],
    "search": ["ctrl+f"],
    "command_palette": ["ctrl+p", "ctrl+P", "/"],
    "cancel": ["escape"],
    "kill_process": ["k"],
    "process_up": ["up"],
    "process_down": ["down"],
    "cycle_sort": ["s"]
  },
  "style": {
    "charset": "braille",
    "borders": "native",
    "pack": null,
    "widgets": {}
  },
  "disabled_plugins": []
}</code></pre>

<p>Unknown keys and missing optional keys are ignored; a file that cannot be
parsed falls back to the defaults above. Existing configs that bind `/` to
`search` are migrated in memory to `command_palette` on load; the next save
persists the new binding.</p>

<hr>

<h2 id="alert-thresholds">Alert Thresholds</h2>

<p>When a metric exceeds its configured threshold, the corresponding widget
changes color to red and displays a warning indicator in its title.</p>

<table>
  <thead>
    <tr>
      <th>Key</th>
      <th>Description</th>
      <th>Default</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><code>cpu_high</code></td>
      <td>CPU usage percentage that triggers a warning</td>
      <td>90.0</td>
    </tr>
    <tr>
      <td><code>mem_high</code></td>
      <td>Memory usage percentage that triggers a warning</td>
      <td>90.0</td>
    </tr>
    <tr>
      <td><code>disk_high</code></td>
      <td>Disk usage percentage that triggers a warning</td>
      <td>90.0</td>
    </tr>
  </tbody>
</table>

<hr>

<h2 id="keybindings">Keybindings</h2>

<p>Every action accepts a list of key strings; the first matching key wins.
Keys are written as: single characters (<code>"q"</code>, <code>"/"</code>, <code>"?"</code>),
shifted characters (<code>"T"</code>, <code>"F"</code>), modified keys
(<code>"ctrl+p"</code>, <code>"alt+x"</code>) and named keys
(<code>"escape"</code>, <code>"enter"</code>, <code>"backspace"</code>, <code>"tab"</code>,
<code>"up"</code>, <code>"down"</code>, <code>"left"</code>, <code>"right"</code>,
<code>"delete"</code>, <code>"home"</code>, <code>"end"</code>, <code>"pageup"</code>,
<code>"pagedown"</code>).</p>

<table>
  <thead>
    <tr>
      <th>Key (action)</th>
      <th>Default binding</th>
      <th>Action</th>
    </tr>
  </thead>
  <tbody>
    <tr><td><code>quit</code></td><td><code>["q"]</code></td><td>Save config and quit</td></tr>
    <tr><td><code>help</code></td><td><code>["?"]</code></td><td>Toggle the help overlay</td></tr>
    <tr><td><code>next_theme</code></td><td><code>["t"]</code></td><td>Next theme</td></tr>
    <tr><td><code>prev_theme</code></td><td><code>["T"]</code></td><td>Previous theme</td></tr>
    <tr><td><code>next_layout</code></td><td><code>["l"]</code></td><td>Next layout</td></tr>
    <tr><td><code>toggle_fullscreen</code></td><td><code>["f"]</code></td><td>Toggle full-screen view</td></tr>
    <tr><td><code>cycle_fullscreen</code></td><td><code>["F"]</code></td><td>Cycle the full-screen widget</td></tr>
    <tr><td><code>search</code></td><td><code>["ctrl+f"]</code></td><td>Start process filtering</td></tr>
    <tr><td><code>command_palette</code></td><td><code>["ctrl+p", "ctrl+P", "/"]</code></td><td>Open the command bar (ctrl+p and / also work as hardcoded fallbacks)</td></tr>
    <tr><td><code>cancel</code></td><td><code>["escape"]</code></td><td>Cancel search / close overlays</td></tr>
    <tr><td><code>kill_process</code></td><td><code>["k"]</code></td><td>Kill the selected process (same-user safety check)</td></tr>
    <tr><td><code>process_up</code></td><td><code>["up"]</code></td><td>Move the process selection up</td></tr>
    <tr><td><code>process_down</code></td><td><code>["down"]</code></td><td>Move the process selection down</td></tr>
    <tr><td><code>cycle_sort</code></td><td><code>["s"]</code></td><td>Cycle the process sort column (CPU% → Memory → PID → Name)</td></tr>
  </tbody>
</table>

<hr>

<h2 id="style">Style</h2>

<p>The <code>style</code> object controls widget glyphs. Values are the
ecosystem-wide enums from <code>xtop-widget-api</code>:</p>

<ul>
  <li><code>charset</code>: <code>braille</code> (default), <code>dot</code>, <code>block</code>, <code>half_block</code>, <code>bar</code> &mdash; chart markers used by history charts.</li>
  <li><code>borders</code>: <code>native</code> (default, classic single-line frame), <code>rounded</code>, <code>double</code>, <code>plain</code>, <code>ascii</code> (<code>plain</code>/<code>ascii</code> draw a pure ASCII <code>+-|</code> frame).</li>
  <li><code>pack</code>: widget pack used for every widget without a per-widget override (<code>"default"</code> or <code>"blocks"</code> when compiled with the <code>widget-blocks</code> feature).</li>
  <li><code>widgets</code>: map of widget-name overrides, each accepting <code>charset</code>, <code>borders</code> and <code>pack</code>.</li>
</ul>

<p>See <a href="customization.md#glyph-style">customization.md</a> for an
example and the full per-widget semantics.</p>

<hr>

<h2 id="custom-themes-and-layouts">Custom Themes and Layouts</h2>

<p>For custom themes and layouts, see the <a href="customization.md">customization guide</a>.</p>

<hr>

<p align="center">
  <a href="../README.md">Back to README</a>
</p>
