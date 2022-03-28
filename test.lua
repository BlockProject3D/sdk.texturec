function main(x, y)
    local texture = Parameters:get("base");
    local pos = Vec2(x, y) / Vec2(Buffer.width, Buffer.height);
    if (Buffer.format ~= format.RGBA8) then
        error("This script only supports 8 bit RGBA!")
    end
    return texture:sample(pos):normalize()
end
