workspace {

    #!identifiers hierarchical

    model {
        user = person "User"
        system_cillio = softwareSystem "cillio" {
            container_cillio_cli = container "CLI" {
                cillio_cli = component "CLI" "Command Line Interface"
            }
            container_cillio_libs = container "Cillio Libs" {
                cillio_config = component "Config" "Graph Config Parser and Validator"
                cillio_graph = component "Graph" "Graph Data Structure"
                cillio_nodes = component "Nodes" "Node Data Structure"
                cillio_runtime = component "Runtime" "Graph Execution Engine"
            }
        }

        user -> cillio_cli "Uses"
        cillio_cli -> cillio_config "Parses config"
        cillio_cli -> cillio_graph "Create graph"
        cillio_cli -> cillio_runtime "Executes graph"

    }

    views {
        systemContext system_cillio "Diagram1" {
            include *
            autoLayout
        }

        container system_cillio "Diagram2" {
            include *
            autoLayout
        }

        component container_cillio_libs "Diagram3" {
            include *
            autoLayout
        }

        image cillio_runtime "Sequence-Diagram" {
            image compute_graph.svg
            title "CLI Invocation"
        }

        styles {
            element "Person" {
                shape person
            }
        }
    }

    configuration {
        scope softwaresystem
    }

}