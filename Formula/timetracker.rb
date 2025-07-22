class Timetracker < Formula
  desc "Cross-platform CLI tool for tracking application window usage time with intelligent activity detection"
  homepage "https://github.com/geraldpeng6/timetracker"
  version "0.2.2"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/geraldpeng6/timetracker/releases/download/v#{version}/timetracker-macos-aarch64.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_ARM64"
    else
      url "https://github.com/geraldpeng6/timetracker/releases/download/v#{version}/timetracker-macos-x86_64.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_X86_64"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/geraldpeng6/timetracker/releases/download/v#{version}/timetracker-linux-aarch64.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_LINUX_ARM64"
    else
      url "https://github.com/geraldpeng6/timetracker/releases/download/v#{version}/timetracker-linux-x86_64.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_LINUX_X86_64"
    end
  end

  def install
    bin.install "timetracker"
    
    # Install documentation
    doc.install "README.md" if File.exist?("README.md")
    doc.install "LICENSE" if File.exist?("LICENSE")
    doc.install "docs/ACTIVITY_DETECTION.md" if File.exist?("docs/ACTIVITY_DETECTION.md")
    
    # Create man page directory and install if available
    man1.mkpath
    
    # Generate shell completions if available
    generate_completions_from_executable(bin/"timetracker", "completion", shells: [:bash, :zsh, :fish])
  end

  def caveats
    <<~EOS
      TimeTracker requires accessibility permissions on macOS to monitor window activity.
      
      To grant permissions:
      1. Open System Preferences > Security & Privacy > Privacy
      2. Select "Accessibility" from the left sidebar
      3. Click the lock to make changes
      4. Add Terminal or your terminal application to the list
      
      For more information, run:
        timetracker permissions check
        
      To get started:
        timetracker start
        timetracker tui
    EOS
  end

  test do
    # Test basic functionality
    assert_match version.to_s, shell_output("#{bin}/timetracker --version")
    
    # Test help command
    assert_match "TimeTracker", shell_output("#{bin}/timetracker --help")
    
    # Test activity detection commands
    assert_match "活跃度检测", shell_output("#{bin}/timetracker activity --help")
    
    # Test configuration
    assert_match "配置", shell_output("#{bin}/timetracker activity config")
  end
end
