using DataStructures
using Distributions
using StableRNGs
using Printf
using Dates
using StatsBase


abstract type Event end

mutable struct procedure
    id::Float64
    room_id::room
    duration::Float64
    delay::Float64
    start_time::Float64
    end_time::FLoat64
end

struct surgeon
    id::Int
    name::String
    availability::Int
end

mutable struct patient
    id::Int
    age::Int
    male::Bool
    priority::Int
    urgency::Int
    surgery_time::Float64
    surgeon_id::Float64
end

mutable struct room_turnover
    id::Int
    duration::Float64
    start_time::Float64
    end_time::Float64
end

mutable struct schedule
    # list of procedures to be performed (Surgeon, Patient, Room, Procedure, Start Time)
    surgical_list::String
end

mutable struct room
    id::Int
    surgical_queue::PriorityQueue{Float64,procedure}
end

mutable struct State
    time::Float64
    surgeons::Vector{surgeon}
    rooms::Vector{room}
end



function State()
    return State(0.0, Vector{surgeon}(), Vector{room_turnover}(), schedule(Vector{procedure}()))
end

function add_surgeon!(state::State, surgeon::surgeon)
    push!(state.surgeons, surgeon)
end

function add_room!(state::State, room::room)
    push!(state.rooms, room)
end


