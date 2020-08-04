mp_state = {
    str = "i'm str",
    num = 233,
    table = {
        a = 1,
        b = false,
        table2 = {
            haha = "hua q",
        },
    }
}

mp_selection = {
    {
        text = "add_num",
        callback = function()
            state.num = state.num + 1
        end
    }
}

mp_show = {
    -- {
    --     type = mp.EShow.Rect,
    --     pos = { x = 4, y = 4 },
    -- }
}

function update(delta)
    -- mp_state.num = math.random(0, 100)
end
