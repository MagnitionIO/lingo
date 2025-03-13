
#include <MagnitionReactorSDK/magnition-reactor-cpp.hh>
using namespace std;


class Main : public MagnitionReactor {
public:

    struct Parameters : public MagnitionParameter<int> {
        
        int test_param = 0;

        Parameters(const std::string &name, MagnitionSimulator *env, int test_param = 10)
            :   MagnitionParameter<int>(name, env),
                test_param(test_param) {

            add_to_map(this->test_param, "test_param",
                ParameterMetadata{
                    .description = "This is test parameter",
                    .min_value = 0,
                    .max_value = 100
                });
        }

        Parameters(const std::string &name, MagnitionReactor *container, int test_param = 10)
            :   MagnitionParameter<int>(name, container),
                test_param(test_param) {

            add_to_map(this->test_param, "test_param",
                ParameterMetadata{
                    .description = "This is test parameter",
                    .min_value = 0,
                    .max_value = 100
                });
        }
    };
private:
    std::unique_ptr<Parameters> parameters;
    const int& test_param = parameters->test_param;
public:                                                         
    Main(const std::string &name, MagnitionSimulator *env, std::unique_ptr<Parameters>&& _params)
        : MagnitionReactor(name, env, _params.get()), parameters(std::move(_params)) {}
    Main(const std::string &name, MagnitionReactor *container, std::unique_ptr<Parameters>&& _params)
        : MagnitionReactor(name, container, _params.get()), parameters(std::move(_params)) {}
                                                                
    int test_local = 0;

    void construct() {
        reaction("reaction_1").
            inputs(&startup).
            outputs().
            function(
                [&](startup_t& startup) {
                    cout << "(" << get_elapsed_logical_time() << ", " << get_microstep() << "), physical_time: " << get_elapsed_physical_time() << " " <<
                    "Starting up reaction Bank:" << bank_index << " name:" << name() << " fully_qualified_name:" << fqn() <<
                    " test_param:" << test_param << " test_local:" << test_local << endl;
                }
        );
    }
};

int main(int argc, char **argv) {
    unsigned workers = 1;
    bool fast{false};
    reactor::Duration timeout = reactor::Duration::max();

    bool visualize = false;

    if (argc > 1) {
        string v_str = argv[1];
        visualize =  (v_str == "true") ? true : false;
    }

    std::cout << "parameters - workers:" << workers << " fast:" << (fast ? "True" : "False") << " timeout:" << timeout << " visualize:" << visualize << std::endl;

    auto *sim = MagnitionSimulator::create_simulator_instance (workers, fast, timeout, visualize);
    Main *hello = new Main("Hello", sim, std::make_unique<Main::Parameters>("Hello", sim));

    sim->run();
  return 0;
}
