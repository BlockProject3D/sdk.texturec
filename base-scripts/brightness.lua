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

-- The brightness multiplier.
local brightness = Parameters:get("brightness")
if brightness == nil then brightness = 1.0 end

local CONVERSIONS = {
    [format.L8] = function(texel)
        return math.int(texel.x * 255)
    end,
    [format.LA8] = function(texel)
        return math.int(texel.x * 255), math.int(texel.w * 255)
    end,
    [format.RGBA8] = function(texel)
        return texel
    end,
    [format.RGBAF32] = function(texel)
        return texel
    end,
    [format.F32] = function(texel)
        return texel.x
    end
}

function main(x, y)
    local texel = Previous:get(x, y):normalize()
    local alpha = texel.w
    texel = texel * Vec4(brightness)
    texel.x = math.clamp(texel.x, 0, 1)
    texel.y = math.clamp(texel.y, 0, 1)
    texel.z = math.clamp(texel.z, 0, 1)
    texel.w = alpha
    return CONVERSIONS[Buffer.format](texel)
end
