class Timetracker < Formula
  desc "Cross-platform CLI tool for tracking application window usage time"
  homepage "https://github.com/yourusername/timetracker"
  version "0.2.0"
  
  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/yourusername/timetracker/releases/download/v#{version}/timetracker-macos-x86_64"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_FOR_INTEL_MAC"
    elsif Hardware::CPU.arm?
      url "https://github.com/yourusername/timetracker/releases/download/v#{version}/timetracker-macos-aarch64"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_FOR_ARM_MAC"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/yourusername/timetracker/releases/download/v#{version}/timetracker-linux-x86_64"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_FOR_LINUX_INTEL"
    elsif Hardware::CPU.arm?
      url "https://github.com/yourusername/timetracker/releases/download/v#{version}/timetracker-linux-aarch64"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_FOR_LINUX_ARM"
    end
  end

  def install
    bin.install "timetracker-#{OS.kernel_name.downcase}-#{Hardware::CPU.arch}" => "timetracker"
  end

  test do
    system "#{bin}/timetracker", "--version"
  end

  def caveats
    <<~EOS
      TimeTracker has been installed successfully!
      
      To get started:
      1. Run 'timetracker permissions' to check and request necessary permissions
      2. Run 'timetracker start' to begin time tracking
      3. Run 'timetracker stats' to view statistics
      
      For more information, visit: https://github.com/yourusername/timetracker
    EOS
  end
end