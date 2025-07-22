#!/usr/bin/osascript

# 基于AppleScript的窗口监测脚本
# 参考: https://gist.github.com/timpulver/4753750
# 使用Accessibility API获取准确的窗口信息

global frontApp, frontAppName, windowTitle

set windowTitle to ""
set bundleId to ""

try
    tell application "System Events"
        # 获取前台应用程序
        set frontApp to first application process whose frontmost is true
        set frontAppName to name of frontApp
        
        # 尝试获取bundle identifier
        try
            set bundleId to bundle identifier of frontApp
        on error
            set bundleId to frontAppName
        end try
        
        # 获取窗口标题
        tell process frontAppName
            try
                # 使用AXMain属性获取主窗口
                tell (1st window whose value of attribute "AXMain" is true)
                    set windowTitle to value of attribute "AXTitle"
                end tell
            on error
                # 如果没有主窗口，尝试获取第一个窗口
                try
                    if (count of windows) > 0 then
                        set windowTitle to name of window 1
                    end if
                on error
                    set windowTitle to "Unknown Window"
                end try
            end try
        end tell
    end tell
    
    # 如果窗口标题为空，使用应用名称
    if windowTitle is "" then
        set windowTitle to frontAppName
    end if
    
    # 返回JSON格式的结果
    return "{\"app_name\":\"" & frontAppName & "\",\"window_title\":\"" & windowTitle & "\",\"bundle_id\":\"" & bundleId & "\"}"
    
on error errorMessage
    # 错误处理
    return "{\"error\":\"" & errorMessage & "\"}"
end try