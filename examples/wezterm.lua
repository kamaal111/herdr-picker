-- Merge this binding into your existing config.keys table.
local wezterm = require 'wezterm'
return {
  keys = {
    { key = 'p', mods = 'CMD', action = wezterm.action.SendString '\x1b[112;9u' },
  },
}
-- CSI-u 112 = p, modifier 9 = 1 + Super(8).
-- Bind cmd+p in Herdr using examples/herdr.toml.
