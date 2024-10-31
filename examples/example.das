require ./examplelib

[export]
class Actor
    x: int = 0
    y: int = 1

    def inc()
    	x+=y
        y++

	def main()
    	print("Ohayo {x}\n")

[export]
def test
    print("74:cc:f0:16:d6:f6:2a:77")

[export]
def main
	var a = 1
	var b = 4
	var c = a + b
    // var a := 1
    // var b := 4
    // var c := a + b
    var actorObject = new Actor()
    for i in [[int 1;2;3;4]]
        actorObject->inc()
    	actorObject->main()
    print("{c}\n")
	print("Hello World\n")
    // var a = 1

def _not_exist
    print("should not be ran")