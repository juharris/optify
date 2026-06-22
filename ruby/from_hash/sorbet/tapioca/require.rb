# typed: true
# frozen_string_literal: true

# Add your extra requires here (`bin/tapioca require` can be used to bootstrap this list)
begin
  require 'rubydex'
rescue LoadError
  # rubydex is platform-specific and may be unavailable on macOS.
end
