use std::fs;
use std::path::Path;

pub fn new(file_name: &str) -> std::io::Result<()> {
    let path = Path::new(file_name);
    if path.exists() {
        println!("file exists! exiting");
        return Ok(());
    }
    let file_content = r#"
mp_state = {}
mp_selection =
{
    {
        text = "hello",
        callback = function()
            mp_state.str = "hello maple"
        end,
    }
}
function update(delta, time_since_start)
    mp_state.delta = delta
    mp_state.time_since_start = time_since_start
    mp_state.fps = math.floor(1 / delta)
end

function awake()
    print("awake")
end
"#;

    fs::write(file_name, file_content)?;
    Ok(())
}
