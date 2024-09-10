-- TODO: This is reused from another lua script
local function deletePattern(pattern)
  local cursor = 0
  repeat
    local result = redis.call('SCAN', cursor, 'MATCH', pattern)
    for _, key in ipairs(result[2]) do
      redis.call('DEL', key)
    end
    cursor = tonumber(result[1])
  until cursor == 0
end

local function mysplit(inputstr, sep)
  if sep == nil then
    sep = "%s"
  end
  local t = {}
  for str in string.gmatch(inputstr, "([^"..sep.."]+)") do
    table.insert(t, str)
  end
  return t
end

local function iterateKeysPattern(pattern, callback)
  local cursor = 0
  repeat
    local next_cursor, keys = unpack(redis.call('SCAN', cursor, 'MATCH', pattern))

    for _, key in ipairs(keys) do
      callback(key)
    end

    cursor = tonumber(next_cursor)
  until cursor == 0
end

local function slotLookupKeyFromIndex(prefix, index)
  return prefix .. ':' .. index .. ':slot_lookup'
end

local function deleteSlotFromSlotLookup(prefix, index, slot)
  redis.call('ZREM', slotLookupKeyFromIndex(prefix, index), slot)
end

local function deleteData(prefix, index, slot)
  redis.call('DEL', prefix .. ':' .. index .. ':' .. slot)
end

local function main()
  local base_slot = tonumber(redis.call('GET', 'base_slot'))
  if base_slot == nil then return 'key not a number: `base_slot`' end

  local last_computed_slot = tonumber(redis.call('GET', 'last_computed_slot'))
  if last_computed_slot == nil then return 'key not a number: `last_computed_slot`' end

  local recompute_slot = tonumber(ARGV[1])
  if recompute_slot == nil then return 'argument not a number: `recompute_slot`' end

  if recompute_slot < base_slot then
    return '`recompute_slot` should not be less than `base_slot`'
  end

  if recompute_slot > last_computed_slot then
    return '`recompute_slot` is not computed'
  end

  local prefixes = { 'validator', 'validator_proof' }

  for _, prefix in ipairs(prefixes) do
    local slot_lookup_key_pattern = prefix .. ':*:slot_lookup'

    iterateKeysPattern(slot_lookup_key_pattern, function(key)
      local parts = mysplit(key, ':')
      local index = tonumber(parts[2])

      local slot_lookup_key = slotLookupKeyFromIndex(prefix, index)
      local slots_to_delete = redis.call('ZRANGE', slot_lookup_key, recompute_slot, 'inf', 'BYSCORE')

      for _, slot in ipairs(slots_to_delete) do
        deleteSlotFromSlotLookup(prefix, index, slot)
        deleteData(prefix, index, slot)
      end
    end)
  end

  deletePattern('validator_proof_queue:*')
  redis.call('SET', 'last_processed_slot', recompute_slot - 1)
  redis.call('SET', 'last_computed_slot', recompute_slot - 1)

  iterateKeysPattern('validators_root:*', function(key)
    local parts = mysplit(key, ':')
    local slot = tonumber(parts[2])
    if slot >= recompute_slot then
      redis.call('DEL', key)
    end
  end)

  return 'Success'
end

return main()
