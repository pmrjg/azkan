//
// Created by pmrj on 23-12-2024.
//

#pragma once

#define GLFW_INCLUDE_VULKAN
#include <GLFW/glfw3.h>
#include <string>

namespace mve {
    class MveWindow {
    public:
        MveWindow(int w, int h, std::string name);
        ~MveWindow();

        MveWindow(const MveWindow &) = delete;
        MveWindow &operator=(const MveWindow &) = delete;

        [[nodiscard]] bool shouldClose() const;

        void createWindowSurface(VkInstance instance, VkSurfaceKHR *surface) const;

    private:
        [[nodiscard]] GLFWwindow* initWindow() const;

        GLFWwindow* window;
        std::pmr::string windowName;
        const int width;
        const int height;

    };
}

