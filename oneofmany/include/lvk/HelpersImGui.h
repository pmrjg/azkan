/*
* LightweightVK
*
* This source src is licensed under the MIT license found in the
* LICENSE file in the root directory of this source tree.
*/

#pragma once

#define IMGUI_DEFINE_MATH_OPERATORS

#include <lvk/LVK.h>
#include <imgui/imgui.h>

namespace lvk {

class ImGuiRenderer {
 public:
  explicit ImGuiRenderer(lvk::IContext& device, const char* defaultFontTTF = nullptr, float fontSizePixels = 24.0f);
  ~ImGuiRenderer();
  
  void updateFont(const char* defaultFontTTF, float fontSizePixels);

  void beginFrame(const lvk::Framebuffer& desc);
  void endFrame(lvk::ICommandBuffer& cmdBuffer);
  
  void setDisplayScale(float displayScale);

 private:
  lvk::Holder<lvk::RenderPipelineHandle> createNewPipelineState(const lvk::Framebuffer& desc);

 private:
  lvk::IContext& ctx_;
  lvk::Holder<lvk::ShaderModuleHandle> vert_;
  lvk::Holder<lvk::ShaderModuleHandle> frag_;
  lvk::Holder<lvk::RenderPipelineHandle> pipeline_;
  lvk::Holder<lvk::TextureHandle> fontTexture_;
  lvk::Holder<lvk::SamplerHandle> samplerClamp_;
  float displayScale_ = 1.0f;

  uint32_t frameIndex_ = 0;

  struct DrawableData {
    lvk::Holder<BufferHandle> vb_;
    lvk::Holder<BufferHandle> ib_;
    uint32_t numAllocatedIndices_ = 0;
    uint32_t numAllocatedVerteices_ = 0;
  };

  DrawableData drawables_[3] = {};
};

} // namespace lvk
