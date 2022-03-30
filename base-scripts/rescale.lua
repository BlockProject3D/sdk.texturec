local CONVERSION_NAMES = {
    [format.L8] = "L8",
    [format.LA8] = "LA8",
    [format.RGBA8] = "RGBA8",
    [format.RGBAF32] = "RGBAF32",
    [format.F32] = "F32"
}

local function rgbaOrError(texel, format)
    local r, g, b, a = texel:rgba()
    if (r == nil) then
        error("Conversion from floats to " .. CONVERSION_NAMES[format] .. " is not supported.")
    end
    return r, g, b, a
end

local CONVERSIONS = {
    [format.L8] = function(texel)
        local r, _ = rgbaOrError(texel, format.L8)
        return r
    end,
    [format.LA8] = function(texel)
        local r, _, _, a = rgbaOrError(texel, format.LA8)
        return r, a
    end,
    [format.RGBA8] = function(texel)
        return rgbaOrError(texel, format.RGBA8)
    end,
    [format.RGBAF32] = function(texel)
        return texel:normalize()
    end,
    [format.F32] = function(texel)
        return texel:normalize().r
    end
}

local function convert(texel, from, to)
    if (from == to) then
        return texel
    end
    return CONVERSIONS[to](texel)
end

function main(x, y)
    local texture = Parameters:get("base");
    if (texture:width() == Buffer.width and texture:height() == Buffer.height) then
        return convert(texture:get(x, y))
    else
        local pos = Vec2(x, y) / Vec2(Buffer.width, Buffer.height)
        return convert(texture:sample(pos))
    end
end
