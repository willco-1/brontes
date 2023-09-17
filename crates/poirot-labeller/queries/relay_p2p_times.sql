SELECT max(relays.timestamp) as relay_timestamp, max(cb.timestamp) as p2p_timestamp
FROM ethereum.relays 
INNER JOIN ethereum.chainbound_block_observations_remote as cb
ON ethereum.relays.block_number = cb.block_number
WHERE  block_number = ? AND block_hash = ?