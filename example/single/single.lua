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
            mp_state.num = mp_state.num + 1
        end
    },
    {
        text = "sub num 2",
        callback = function()
            mp_state.num = mp_state.num - 2
        end
    },
    {
        text = "error",
        callback = function()
            not_exist.field = 3
        end,
    },
    {
        text = "insert",
        callback = function()
            mp_state[math.random(1, 10)] = math.random(1, 10)
        end,
    }
}

mp_show = {
    -- {
    --     type = mp.EShow.Rect,
    --     pos = { x = 4, y = 4 },
    -- }
}

function update(delta, time_since_start)
    -- mp_state.num = math.random(0, 100)
    -- print(delta, time_since_start)
    mp_state.delta = delta
    mp_state.time_since_start = time_since_start
    mp_state.fps = math.floor(1 / delta)
end

function awake()
    print("awake")
end