-- bot_script.lua - Example Lua bot script

function on_join()
    print("Bot joined.")
end

function on_chat(sender, text)
    if text == "!pos" then
        local x, y, z = bot.get_pos()
        bot.chat(string.format("Position: (%.1f, %.1f, %.1f)", x, y, z))
    elseif text == "!hp" then
        local hp = bot.get_hp()
        bot.chat("HP: " .. hp)
    elseif text == "!help" then
        bot.chat("Commands: !pos, !hp, !jump, !stop, !quit")
    elseif text == "!quit" then
        bot.chat("Stopping.")
        bot.disconnect()
    end
end

function on_death()
    --print("Bot dead")
end

function on_hp(hp)
    --bot.chat(hp)
end

function on_tick()
    -- Optional: Custom movement logic here
    -- Example: Walk forward continuously
    -- bot.walk(true, false, false, false)
end

function on_kick(reason)
    print("Kicked: " .. reason)
end

function on_disconnect()
    print("Disconnected from server")
end