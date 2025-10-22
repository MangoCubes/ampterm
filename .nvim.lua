-- Disable mouse
vim.opt.mouse = ""

local root_patterns = { ".git", ".clang-format", "pyproject.toml", "setup.py" }
local proj_dir = vim.fs.dirname(vim.fs.find(root_patterns, { upward = true })[1])

local function run_cargo()
	vim.cmd('silent !kitty sh -c "(source ' .. proj_dir .. '/.envrc && cargo run) || read" &')
end

vim.keymap.set('n', '<M-s>', run_cargo, { noremap = true, silent = true })
vim.keymap.set('n', '<M-C-s>', run_cargo, { noremap = true, silent = true })

local dap = require("dap")

dap.configurations.rust = {
	{
		name = "Debug Ampterm",
		type = "lldb",
		request = "launch",
		program = function()
			return proj_dir .. "/target/debug/ampterm"
		end,
		cwd = "${workspaceFolder}",
		stopOnEntry = false,
		runInTerminal = true,
	}
}
