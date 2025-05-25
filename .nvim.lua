-- Disable mouse
vim.opt.mouse = ""
-- Alt+s runs cargo
vim.fn.sign_define("DapBreakpoint", { text = "🔴" })

local dap = require("dap")

local function get_executable_path(executable_name)
	local handle = io.popen("whereis " .. executable_name)
	if handle == nil then
		error(executable_name .. " is not installed! Please install it.")
	end
	local result = handle:read("*a")
	handle:close()

	local paths = {}
	for path in result:gmatch("%S+") do
		table.insert(paths, path)
	end

	-- The first entry in paths is the executable name, the rest are the paths
	if #paths > 1 then
		return paths[2] -- Return the first path found
	else
		return nil -- Executable not found
	end
end

local ldap = get_executable_path("lldb-dap")

local root_patterns = { ".git", ".clang-format", "pyproject.toml", "setup.py" }
local proj_dir = vim.fs.dirname(vim.fs.find(root_patterns, { upward = true })[1])

local function run_cargo()
	vim.cmd('silent !kitty sh -c "(source ' .. proj_dir .. '/.envrc && cargo run) || read" &')
end

local function run_cargo_release()
	vim.cmd('silent !kitty sh -c "(source ' .. proj_dir .. '/.envrc && cargo run --profile release) || read" &')
end
vim.keymap.set('n', '<M-s>', run_cargo, { noremap = true, silent = true })
vim.keymap.set('n', '<M-C-s>', run_cargo, { noremap = true, silent = true })

dap.adapters.lldb = {
	type = "executable",
	command = ldap, -- adjust as needed
	name = "lldb",
}

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


local dapui = require("dapui")
dapui.setup()

dap.listeners.before.attach.dapui_config = function()
	dapui.open()
end
dap.listeners.before.launch.dapui_config = function()
	dapui.open()
end
dap.listeners.before.event_terminated.dapui_config = function()
	dapui.close()
end
dap.listeners.before.event_exited.dapui_config = function()
	dapui.close()
end
