require "./state"

mp_state = {
    a = state.a,
    c = state.c,
}

mp_selection = {
    {
        text = "run time error",
        callback = function()
            mp_state["d"] = "233"
            a.y = 4
        end,
    }
}

function update()
    -- print("update")
end