-- Copyright (c) 2022, BlockProject 3D
--
-- All rights reserved.
--
-- Redistribution and use in source and binary forms, with or without modification,
-- are permitted provided that the following conditions are met:
--
--     * Redistributions of source code must retain the above copyright notice,
--       this list of conditions and the following disclaimer.
--     * Redistributions in binary form must reproduce the above copyright notice,
--       this list of conditions and the following disclaimer in the documentation
--       and/or other materials provided with the distribution.
--     * Neither the name of BlockProject 3D nor the names of its contributors
--       may be used to endorse or promote products derived from this software
--       without specific prior written permission.
--
-- THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
-- "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
-- LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
-- A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR
-- CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
-- EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
-- PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
-- PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
-- LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
-- NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
-- SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

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
        return texel:normalize().x
    end
}

local function convert(texel, from, to)
    if (from == to) then
        return texel
    end
    return CONVERSIONS[to](texel)
end

local texture = Parameters:get("base")
local size = Vec2(Buffer.width, Buffer.height)

function main(x, y)
    if (texture:width() == Buffer.width and texture:height() == Buffer.height) then
        return convert(texture:get(x, y))
    else
        local pos = Vec2(x, y) / size
        return convert(texture:sample(pos))
    end
end
