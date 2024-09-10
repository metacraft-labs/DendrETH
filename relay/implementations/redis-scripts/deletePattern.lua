local cursor = 0
local calls = 0
local dels = 0
repeat
    local result = redis.call('SCAN', cursor, 'MATCH', ARGV[1])
    calls = calls + 1
    for _,key in ipairs(result[2]) do
        redis.call('DEL', key)
        dels = dels + 1
    end
    cursor = tonumber(result[1])
until cursor == 0
return "Calls " .. calls .. " Dels " .. dels
