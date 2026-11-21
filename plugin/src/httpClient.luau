-- StudioPresence HTTP client (ISC, © 2022 iArxic), UltimatePresence fork:
-- "unchanged" now compares the whole activity, not just the top line, so switching
-- between tools that share wording (e.g. two animators, both "Animating") is still sent.

local httpClient = {}
local HTTP = game:GetService("HttpService")

local scriptParent = script.Parent

local function activityKey(requestBody)
	local a = requestBody.activity or {}
	local s = a.assets or {}
	return table.concat({
		tostring(a.details),
		tostring(a.state),
		tostring(s.large_image),
		tostring(s.large_text),
		tostring(s.small_image),
	}, "|")
end

function httpClient.new(url)
	local self = setmetatable({ URL = url }, { __index = httpClient })
	return self
end

function httpClient:Post(requestBody)
	local success, reply = pcall(function()
		if requestBody.updateType ~= "CLOSE" then
			if script:GetAttribute("previous") and script:GetAttribute("previous") == activityKey(requestBody) then
				return "Repeated"
			end
		end
		return HTTP:PostAsync(self.URL, HTTP:JSONEncode(requestBody))
	end)
	if not success then
		if scriptParent.Parent:FindFirstChild("index") then
			local AttributeValue = scriptParent.Parent.index:GetAttribute("Disabled")

			if AttributeValue == nil or AttributeValue == false then
				scriptParent.Parent.index:SetAttribute("Disabled", true)
			end

			scriptParent.Parent.index:SetAttribute("Disabled", false)
		end
	end

	if success and requestBody.updateType ~= "CLOSE" then
		reply = activityKey(requestBody)
	end
	return success, reply
end

return httpClient
