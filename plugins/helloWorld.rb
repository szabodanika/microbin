module MBP
	class HelloWorld < MBPlugin

		def self.get_id()
      "HelloWorld"
    end

		def self.get_name() 
      "Hello World Plugin"
    end

    def self.get_version() 
      "1.0.0"
    end

    def self.get_description() 
      "This is just a demo plugin. It does not do anything."
    end

		def self.get_author() 
      "Daniel Szabo"
    end

    def self.get_webpage() 
      "https://github.com/szabodanika/microbin"
    end

		def self.init() 
      # Ignore event
			"OK"
    end

		def self.on_pasta_created(content)
			return content
		end

		def self.on_pasta_read(content)
			return content
		end

	end
end
