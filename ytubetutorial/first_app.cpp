//
// Created by pmrj on 23-12-2024.
//

#include "first_app.h"

#include <ranges>
#include <stdexcept>
#define GLFW_INCLUDE_VULKAN
#include <GLFW/glfw3.h>


namespace mve {
    FirstApp::FirstApp() {
        createPipelineLayout();
        createPipeline();
        createCommandBuffers();
    }

    FirstApp::~FirstApp() {
        vkDestroyPipelineLayout(mveDevice.device(), pipelineLayout, nullptr);
    }

    void FirstApp::run() {
        while (!mveWindow.shouldClose()) {
            glClear(GL_COLOR_BUFFER_BIT);
            glfwSwapBuffers(mveWindow.getSurface());
            glfwPollEvents();
        }
    }

    void FirstApp::createPipelineLayout() {
        VkPipelineLayoutCreateInfo pipelineLayoutInfo{};
        pipelineLayoutInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO;
        pipelineLayoutInfo.setLayoutCount = 0;
        pipelineLayoutInfo.pSetLayouts = nullptr;
        pipelineLayoutInfo.pushConstantRangeCount = 0;
        pipelineLayoutInfo.pPushConstantRanges = nullptr;

        if (vkCreatePipelineLayout(mveDevice.device(), &pipelineLayoutInfo, nullptr, &pipelineLayout) != VK_SUCCESS) {
            throw std::runtime_error("failed to create pipeline layout");
        }
    }

    void FirstApp::createPipeline() {
        auto pipelineConfig = MvePipeline::defaultPipelineConfigInfo(mveSwapChain.width(), mveSwapChain.height());
        pipelineConfig.renderPass = mveSwapChain.getRenderPass();
        pipelineConfig.pipelineLayout = pipelineLayout;

        mvePipeline = std::make_unique<MvePipeline>(mveDevice, "shaders/simple_shader.vert.spv",
                                                    "shaders/simple_shader.frag.spv", pipelineConfig);
    }
    void FirstApp::createCommandBuffers() {
        commandBuffers.resize(mveSwapChain.imageCount());

        VkCommandBufferAllocateInfo allocInfo{};
        allocInfo.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO;
        allocInfo.level = VK_COMMAND_BUFFER_LEVEL_PRIMARY;
        allocInfo.commandPool = mveDevice.getCommandPool();
        allocInfo.commandBufferCount = static_cast<uint32_t>(commandBuffers.size());

        if (vkAllocateCommandBuffers(mveDevice.device(), &allocInfo, commandBuffers.data()) != VK_SUCCESS) {
            throw std::runtime_error("failed to allocate command buffers");
        }

        for (size_t i : std::ranges::views::iota(0u,commandBuffers.size())) {
            VkCommandBufferBeginInfo beginInfo{};
            beginInfo.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO;

            if (vkBeginCommandBuffer(commandBuffers[i], &beginInfo) != VK_SUCCESS) {
                throw std::runtime_error("failed to begin command buffer operation");
            }

            VkRenderPassBeginInfo renderPassInfo{};
            renderPassInfo.sType = VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO;
            renderPassInfo.renderPass = mveSwapChain.getRenderPass();
            renderPassInfo.framebuffer = mveSwapChain.getFrameBuffer(i);

            renderPassInfo.renderArea.offset = {0, 0};
            renderPassInfo.renderArea.extent = mveSwapChain.getSwapChainExtent();

            std::array<VkClearValue, 2> clearValues{};
            clearValues[0].color = {0.1f, 0.1f, 0.1f, 1.0f};
            clearValues[1].depthStencil = {1.0f, 0};
            renderPassInfo.clearValueCount = static_cast<uint32_t>(clearValues.size());
            renderPassInfo.pClearValues = clearValues.data();

            vkCmdBeginRenderPass(commandBuffers[i], &renderPassInfo, VK_SUBPASS_CONTENTS_INLINE);

            mvePipeline->bind(commandBuffers[i]);
            vkCmdDraw(commandBuffers[i], 3, 1, 0, 0);

            vkCmdEndRenderPass(commandBuffers[i]);
            if (vkEndCommandBuffer(commandBuffers[i]) != VK_SUCCESS) {
                throw std::runtime_error("failed to end command buffer operation");
            }
        }
    }
    void FirstApp::drawFrame() {
        uint32_t imageIndex;
        auto result = mveSwapChain.acquireNextImage(&imageIndex);
        if (result != VK_SUCCESS && result != VK_SUBOPTIMAL_KHR) {
            throw std::runtime_error("failed to acquire swap chain image");
        }

        result = mveSwapChain.submitCommandBuffers(&commandBuffers[imageIndex], &imageIndex);

        if (result != VK_SUCCESS) {
            throw std::runtime_error("failed to submit command buffer operation");
        }
    }

} // namespace mve
