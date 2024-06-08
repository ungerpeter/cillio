workspace {

    #!identifiers hierarchical

    model {
        user = person "User"
        system_cillio = softwareSystem "cillio" {
            container_cillio_cli = container "CLI" {
                cillio_cli = component "CLI" "TBD"
            }
            container_cillio_libs = container "Cillio Libs" {
                cillio_config = component "Config" "TBD"
                cillio_graph = component "Graph" "TBD"
                cillio_nodes = component "Nodes" "TBD"
                cillio_runtime = component "Runtime" "TBD"
            }
        }

        user -> cillio_cli "Uses"
        cillio_cli -> cillio_config "Uses"
        cillio_cli -> cillio_graph "Uses"
        cillio_cli -> cillio_runtime "Uses"

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
    }

    configuration {
        scope softwaresystem
    }

}