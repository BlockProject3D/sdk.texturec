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

-- The sigma parameter.
local sigma = Parameters:get("sigma")
if sigma == nil then sigma = 1.5 end

-- The kernel size.
local ksize = Parameters:get("ksize")
if ksize == nil then ksize = 3 end

if Buffer.width ~= Previous:width() or Buffer.height ~= Previous:height() then
    error("This script only runs as post process from previous buffer to new buffer")
end

local W = Buffer.width
local H = Buffer.height

function main(x, y)
    local gsigma = Vec3(0)
    local w = 0
    local p = Vec2(x, y)
    for i = -ksize, ksize - 1 do
        for j = -ksize, ksize - 1 do
            local q = Vec2(math.clamp(x + j, 0, W - 1), math.clamp(y + i, 0, H - 1))
            local norm = (p - q):normSquared()
            local kernel = math.gaussian2d(sigma, norm)
            local r, g, b = Previous:get(math.int(q.x), math.int(q.y)):rgba();
            gsigma = gsigma + (Vec3(r, g, b) * Vec3(kernel))
            w = w + kernel
        end
    end
    local rgb = gsigma / Vec3(w)
    local r = math.int(rgb.x)
    local g = math.int(rgb.y)
    local b = math.int(rgb.z)
    return r, g, b
end
