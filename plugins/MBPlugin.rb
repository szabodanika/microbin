require 'rutie'

module MB

  class MBPlugin
		# Plugin Properties

    def self.get_id()
      "[Enter Plugin ID]"
    end

    def self.get_name()
      "[Eenter Plugin Name]"
    end

    def self.get_version()
      "1.0.0"
    end

    def self.get_author()
      "[Enter Author name]"
    end

    def self.get_webpage()
      "[Enter Web URL]"
    end

    def self.get_description()
      "[Enter Description]"
    end

		# Plugin Event Hooks

    def self.init()
      raise "Operation not supported";
    end

    def self.on_pasta_deleted(id, content, created, expiration, file)
			raise "Operation not supported";
		end

		def self.on_pasta_expired(id, content, created, expiration, file)
			raise "Operation not supported";
		end

    def self.on_pasta_created(id, content, created, expiration, file)
      raise "Operation not supported";
    end

    def self.on_pasta_read(id, content, created, expiration, file)
      raise "Operation not supported";
    end

    # Rust Function Calls

    def self.init()
      raise "Operation not supported";
    end

    def self.P=on_pasta_deleted(id, content, created, expiration, file)
			raise "Operation not supported";
		end

		def self.on_pasta_expired(id, content, created, expiration, file)
			raise "Operation not supported";
		end

    def self.on_pasta_created(id, content, created, expiration, file)
      raise "Operation not supported";
    end

    def self.on_pasta_read(id, content, created, expiration, file)
      raise "Operation not supported";
    end

  end

end