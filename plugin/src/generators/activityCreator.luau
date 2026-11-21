-- UltimatePresence fork of StudioPresence's activity generator (ISC, © 2022 iArxic).
-- Adds detection for Eclipse Animator and Qwinkle's Part-Icles next to Moon Animator.

local activityCreator = {}

local StudioPresence = script:FindFirstAncestor("StudioPresence")
local CoreGui = game:GetService("CoreGui")

-- Eclipse Animator keeps a ScreenGui named "Eclipse Editor [Animator] <id>" in CoreGui
-- and enables it while the editor is open.
local function eclipseOpen()
	local ok, open = pcall(function()
		for _, gui in CoreGui:GetChildren() do
			if gui:IsA("ScreenGui") and gui.Enabled and string.sub(gui.Name, 1, 14) == "Eclipse Editor" then
				return true
			end
		end
		return false
	end)
	return ok and open
end

-- Qwinkle's Part-Icles (v2) creates CoreGui.PartIclesUI while it's open and removes it when closed.
local function partIclesOpen()
	local ok, open = pcall(function()
		local ui = CoreGui:FindFirstChild("PartIclesUI")
		return ui ~= nil and ui:IsA("ScreenGui") and ui.Enabled
	end)
	return ok and open
end

local Activity = require(StudioPresence.src.activity).new()
local FormatString = require(StudioPresence.src.generators.formatString)

function activityCreator:Get()
	local RunService = game:GetService("RunService")

	local description = "Developing"
	local editing = "UltimatePresence by aqua"
	local imageKey = "studio"
	local smallImageKey, smallImageDesc = "none", "none"
	local lineText

	-- Playtesting mode - StudioService doesn't exist during playtest
	if RunService:IsRunning() then
		editing = "UltimatePresence by aqua"
		description = "Testing"
		imageKey = "studio"
		smallImageKey = "playtesticon"
		smallImageDesc = "Testing"

		Activity:clear()
			:setDescription(description)
			:setState(FormatString:process("Workspace: $WORKSPACE"))
			:setImage(imageKey, editing)
			:setTimer()
			:setThumbnail(smallImageKey, smallImageDesc)

		return Activity
	end

	-- Studio-only services (not available during playtest)
	local StudioService = game:GetService("StudioService")
	local ScriptEditorService = game:GetService("ScriptEditorService")

	local activeScript = StudioService.ActiveScript

	-- Animation and VFX tools are checked independently so they can combine.
	local animator, animatorBadge = nil, nil
	if workspace.Camera:FindFirstChild("MoonAnimator2MouseFilter") then
		animator, animatorBadge = "Moon Animator 2", "moonanimator"
	elseif eclipseOpen() then
		animator, animatorBadge = "Eclipse Animator", "eclipseanimator"
	end
	local vfx = partIclesOpen() and "Qwinkle's Part-Icles" or nil

	if animator and vfx then
		editing = animator .. " + " .. vfx
		description = "Animating & making VFX"
		imageKey = "studio"
		smallImageKey = "animatingvfx"
		smallImageDesc = "Animating & making VFX"
	elseif animator then
		editing = animator
		description = "Animating"
		imageKey = "studio"
		smallImageKey = animatorBadge
		smallImageDesc = "Animating"
	elseif vfx then
		editing = vfx
		description = "Making VFX"
		imageKey = "studio"
		smallImageKey = "qwinkleparticles"
		smallImageDesc = "Making VFX"
	elseif activeScript then
		local doc = ScriptEditorService:FindScriptDocument(activeScript)

		if doc then
			local sLine, _sChar, _eLine, _eChar = doc:GetSelection()
			local cursorLine = sLine or 1
			lineText = doc:GetLine(cursorLine) or ""
			lineText = lineText:gsub("^%s+", "")
			lineText = lineText:sub(1, 120)

			local lineCount = doc:GetLineCount()

			description = string.format("%s (Line %d / %d)", activeScript.Name, cursorLine, lineCount)
		else
			-- Fallback to editor/source line count if document isn't open
			local text = ScriptEditorService:GetEditorSource(activeScript) or activeScript.Source or ""
			local lines = #string.split(text, "\n")
			description = string.format("%s (%d lines)", activeScript.Name, lines)
		end

		-- script type -> image/label (kept from your cleaned version)
		if activeScript:IsA("ModuleScript") then
			editing = "Editing a MODULE Script"
			imageKey = "modulescript"
		elseif activeScript:IsA("LocalScript") then
			editing = "Editing a CLIENT Script"
			imageKey = "clientscript"
		elseif activeScript:IsA("Script") then
			if activeScript.RunContext ~= Enum.RunContext.Client then
				editing = "Editing a SERVER Script"
				imageKey = "serverscript"
			else
				editing = "Editing a CLIENT Script"
				imageKey = "clientscript"
			end
		end
	end

	Activity:clear()
		:setDescription(description)
		:setState(FormatString:process("Workspace: $WORKSPACE"))
		:setImage(imageKey, editing)
		:setTimer()
		:setThumbnail(smallImageKey, smallImageDesc)

	return Activity
end

return activityCreator
