-- UltimatePresence Roblox Studio plugin. Sends what you're doing to the UltimatePresence
-- app on localhost:4455. Based on StudioPresence by iArxic (ISC licence, © 2022 iArxic).
--
-- Original note from StudioPresence:
-- CREDITS TO RIGID STUDIOS FOR MAKING THIS OPEN SOURCED!! THIS ISNT MY ORIGINAL SCRIPT, JUST HEAVILY EDITED!

local scriptParent = script.Parent

scriptParent.src.activity:SetAttribute("started", os.time())
local started = false

local function start()
	local Root = script:FindFirstAncestor("UltimatePresence")
	local env = {
		["plugin"] = plugin,
	}

	for i, v in ipairs(Root:GetDescendants()) do
		if not v:IsA("ModuleScript") then
			continue
		end
		local m = require(v)
		setmetatable(m, { __index = env })
	end

	local Data = require(Root.src.dataHandler)
	local Client = require(Root.src.client.index)

	local Http = require(Root.src.httpClient).new("http://localhost:4455/")
	local ClientObj = Client.new(Http, false)

	Data:AttachChange("Enabled", function(isEnabled)
		if isEnabled then
			ClientObj:Open()
		else
			ClientObj:Close()
		end
	end)
	
	if not plugin then return end
	plugin.Unloading:Connect(function()
		ClientObj.Enabled = false
		ClientObj.Terminated = true
	end)

	ClientObj:login(function(success)
		if success then
			started = true
		else
			task.wait(3)
			start()
			ClientObj.Enabled = false
		end
	end)
end

if not started then
	start()
end
