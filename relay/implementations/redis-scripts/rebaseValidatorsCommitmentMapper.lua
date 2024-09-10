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

local function slotLookupKeyFromIndex(prefix, index)
  return prefix .. ':' .. index .. ':slot_lookup'
end

local function deleteSlotFromSlotLookup(prefix, index, slot)
  redis.call('ZREM', slotLookupKeyFromIndex(prefix, index), slot)
end

local function deleteData(prefix, index, slot)
  redis.call('DEL', prefix .. ':' .. index .. ':' .. slot)
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

local function rebaseEntriesPrefix(state, prefix)
  local slot_lookup_key_pattern = prefix .. ':*:slot_lookup'
  iterateKeysPattern(slot_lookup_key_pattern, function(key)
    local parts = mysplit(key, ':')
    local index = tonumber(parts[2])

    local slot_lookup_key = slotLookupKeyFromIndex(prefix, index)

    local computed_slots = redis.call('ZRANGE', slot_lookup_key, 0, state.anchor_slot, 'BYSCORE')
    local most_recent_computed_slot = computed_slots[#computed_slots]

    local slots_to_delete = state.rebase_slot < state.last_computed_slot
      and computed_slots
      or redis.call('ZRANGE', slot_lookup_key, 0, -1) -- Get all keys

    for _, slot in ipairs(slots_to_delete) do
      if slot ~= most_recent_computed_slot then
        deleteSlotFromSlotLookup(prefix, index, slot)
        deleteData(prefix, index, slot)
      end
    end
  end)
end

local function getValidState()
  local rebase_slot = tonumber(ARGV[1])
  if rebase_slot == nil then return nil, 'argument not a number: `rebase_slot`' end

  local base_slot = tonumber(redis.call('GET', 'base_slot'))
  if base_slot == nil then return nil, 'key not a number: `base_slot`' end

  local last_computed_slot = tonumber(redis.call('GET', 'last_computed_slot'))
  if last_computed_slot == nil then return nil, 'key not a number: `last_computed_slot`' end

  local last_processed_slot = tonumber(redis.call('GET', 'last_processed_slot'))
  if last_processed_slot == nil then return nil, 'key not a number: `last_processed_slot`' end

  if rebase_slot <= base_slot then
    return nil, 'The rebase slot ' .. rebase_slot .. ' must be greater than the current base slot ' .. base_slot
  end

  if last_computed_slot < base_slot then
    return nil, 'Rebase failed. Base slot is not computed.'
  end

  local anchor_slot = math.min(last_computed_slot, rebase_slot)

  return {
    base_slot = base_slot,
    rebase_slot = rebase_slot,
    anchor_slot = anchor_slot,
    last_computed_slot = last_computed_slot,
  }
end

local function main()
  redis.debug('calling "rebaseValidatorsCommitmentMapper"')

  local state, err = getValidState()
  if err ~= nil then return err end

  rebaseEntriesPrefix(state, 'validator')
  rebaseEntriesPrefix(state, 'validator_proof')

  if state.rebase_slot > state.last_computed_slot then
    deletePattern('validator_proof_queue:*')
    redis.call('SET', 'last_processed_slot', state.rebase_slot - 1)
  end

  redis.call('SET', 'base_slot', state.rebase_slot)

  return 'Success'
end

return main()
