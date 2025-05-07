require 'json'
require 'test/unit'
require_relative '../lib/optify'
require_relative 'my_config'

def main
  provider = Optify::OptionsWatcherBuilder.new
                                          .add_directory('../../tests/test_suites/simple/configs')
                                          .build
  last_modified = provider.last_modified

  puts "last_modified: #{last_modified}"

  loop do
    command = 'touch ../../tests/test_suites/simple/configs/feature_A.json'
    puts "Running command: #{command}"
    system(command)
    puts 'File has been touched.'
    STDIN.gets
    last_modified = provider.last_modified
    puts "last_modified: #{last_modified}"
  end
end

main if __FILE__ == $PROGRAM_NAME
